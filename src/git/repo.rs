use std::{env, ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};

use anyhow::{Context, Result};
use git2::{
    Cred, CredentialType, ErrorCode, FetchOptions, FileFavor, IndexAddOption, MergeOptions,
    ObjectType, PushOptions, RemoteCallbacks, Repository, Signature, build::CheckoutBuilder,
};
use log::{debug, info};

use crate::git::MergeStrategy;

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    /// Initialize Git repository, create a new one if it doesn't exist
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        // If the repository doesn't exist, create a new one
        let repo = Repository::open(path).or_else(|_| Repository::init(path))?;
        Ok(GitRepo { repo })
    }

    /// Clone a remote repository to the local directory
    ///
    /// - url: URL of the remote repository
    /// - branch: Branch to clone (default is "main" or "master")
    pub fn clone<P: AsRef<Path>>(path: P, url: &str) -> Result<Self> {
        let path = path.as_ref();
        info!("Cloning {} to {}", url, path.display());

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(credential);

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Prepare builder.
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // Clone the project.
        match builder.clone(url, path) {
            Ok(repo) => {
                info!("Successfully cloned repository");
                Ok(GitRepo { repo })
            }
            Err(e) => {
                if e.to_string()
                    .contains("exists and is not an empty directory")
                {
                    anyhow::bail!(
                        "The target directory already exists and is not empty: {}",
                        path.display()
                    )
                } else {
                    anyhow::bail!("Fail to clone repository: {}", e)
                }
            }
        }
    }

    /// Automatically commit changes
    pub fn commit_changes(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let signature = Signature::now("rutd", "rutd@auto.commit")?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        // Get the current HEAD commit
        let head = match self.repo.head() {
            Ok(head) => Some(head.peel(ObjectType::Commit)?.into_commit().unwrap()),
            Err(_) => None,
        };

        // Create a new commit
        let parents = match head {
            Some(ref commit) => vec![commit],
            None => vec![],
        };
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;

        debug!("Created commit: {}", commit_id);
        Ok(())
    }

    /// Generate commit message using Conventional Commits format
    pub fn generate_commit_message(task_id: &str, action: &str) -> String {
        format!("chore({}): {}", task_id, action)
    }

    /// Sync with remote repository (fetch, pull, push)
    ///
    /// - prefer: 指定在合并冲突时的解决策略
    pub fn sync(&self, prefer: MergeStrategy) -> Result<()> {
        // Check if we have any remotes
        let remotes = self.repo.remotes()?;
        if remotes.is_empty() {
            info!("No remote repository configured. Skipping sync.");
            return Ok(());
        }

        info!("Syncing with remote repository...");

        // Set up authentication callbacks
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(credential);

        // Fetch latest changes
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Get the default remote (usually "origin")
        let remote_name = "origin";

        // Fetch from remote
        debug!("Fetching from remote '{}'", remote_name);
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .context(format!("No remote named '{}' found", remote_name))?;

        // Attempt to fetch but handle the case where the remote is empty or unreachable
        match remote.fetch(&["master", "main"], Some(&mut fetch_options), None) {
            Ok(_) => debug!("Successfully fetched from remote"),
            Err(e) => {
                // Check if this is a fresh/empty repository error
                if e.to_string().contains("couldn't find remote ref") {
                    debug!("Remote repository appears to be empty or the branch doesn't exist yet");
                    // Continue with push only
                } else {
                    // For other errors, return the error
                    return Err(e.into());
                }
            }
        }

        // Get the current branch name
        let head = match self.repo.head() {
            Ok(head) => head,
            Err(e) => {
                // If HEAD is not yet set (no commits), create an initial commit
                if matches!(e.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) {
                    debug!("No HEAD found, repository might be empty");
                    return Ok(());
                }
                return Err(e.into());
            }
        };

        let branch_name = if head.is_branch() {
            head.shorthand().unwrap_or("master")
        } else {
            "master"
        };

        debug!("Current branch: {}", branch_name);

        // Try to merge remote changes
        let remote_branch = format!("refs/remotes/{}/{}", remote_name, branch_name);
        if let Ok(remote_reference) = self.repo.find_reference(&remote_branch) {
            let remote_commit = remote_reference.peel_to_commit()?;
            let annotated_commit = self.repo.find_annotated_commit(remote_commit.id())?;
            let analysis = self.repo.merge_analysis(&[&annotated_commit])?;

            if analysis.0.is_up_to_date() {
                debug!("Local repository is up to date");
            } else if analysis.0.is_fast_forward() {
                debug!("Fast-forwarding local repository");

                // Perform the fast-forward
                let mut reference = self
                    .repo
                    .find_reference(&format!("refs/heads/{}", branch_name))?;
                reference.set_target(remote_commit.id(), "Fast-forward update")?;

                // Update the working directory
                self.repo.set_head(&format!("refs/heads/{}", branch_name))?;
                self.repo
                    .checkout_head(Some(CheckoutBuilder::new().force()))?;

                info!("Successfully pulled changes from remote");
            } else if analysis.0.is_normal() {
                // Need to perform a merge with possible conflicts
                debug!("Merge required - analyzing merge strategy");

                // Create merge options
                let mut merge_opts = MergeOptions::new();
                match prefer {
                    MergeStrategy::None => {
                        // No automatic conflict resolution
                        merge_opts.file_favor(FileFavor::Normal);
                    }
                    MergeStrategy::Local => {
                        // Prefer local changes
                        merge_opts.file_favor(FileFavor::Ours);
                    }
                    MergeStrategy::Remote => {
                        // Prefer remote changes
                        merge_opts.file_favor(FileFavor::Theirs);
                    }
                }

                // Perform the merge
                self.repo
                    .merge(&[&annotated_commit], Some(&mut merge_opts), None)?;

                // Handle merge conflicts based on the prefer option
                let conflicts = self.repo.index()?.conflicts()?.collect::<Vec<_>>();
                if conflicts.is_empty() {
                    debug!("Successfully merged remote changes");
                } else {
                    debug!("Merge conflicts detected");
                }
                for conflict in conflicts {
                    let conflict = conflict?;
                    // Resolve each conflict based on the prefer option
                    let mut index = self.repo.index()?;
                    match prefer {
                        MergeStrategy::Local => {
                            if let Some(ours) = conflict.our {
                                index.conflict_remove(Path::new(&OsStr::from_bytes(&ours.path)))?;
                                index.add_path(Path::new(&OsStr::from_bytes(&ours.path)))?;
                            }
                        }
                        MergeStrategy::Remote => {
                            if let Some(theirs) = conflict.their {
                                index
                                    .conflict_remove(Path::new(&OsStr::from_bytes(&theirs.path)))?;
                                index.add_path(Path::new(&OsStr::from_bytes(&theirs.path)))?;
                            }
                        }
                        MergeStrategy::None => {
                            // Skip automatic resolution, tell the user to resolve manually
                            anyhow::bail!(
                                "Merge conflicts detected. Please resolve them manually. Then continue with 'sync --continue'"
                            )
                        }
                    };
                    index.write()?;
                }

                // Commit the merge
                let commit_message = format!(
                    "Merge remote-tracking branch '{}' into '{}'",
                    remote_branch, branch_name
                );
                self.commit_merge(&annotated_commit, &commit_message)
                    .context("Failed to commit merge")?;
            }
        } else {
            debug!(
                "Remote branch '{}' not found. This might be a new remote repository.",
                remote_branch
            );
        }

        // Push local changes
        debug!("Pushing to remote '{}'", remote_name);
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(credential);
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let mut remote = self.repo.find_remote(remote_name)?;

        // Check if we have any commits to push
        if self.repo.head().is_ok() {
            // Ensure HEAD exists (at least one commit)
            match remote.push(
                &[format!("refs/heads/{}", branch_name)],
                Some(&mut push_options),
            ) {
                Ok(_) => info!("Successfully pushed to remote repository"),
                Err(e) => {
                    if e.to_string().contains("non-fast-forward") {
                        info!(
                            "Cannot push because remote contains work that you do not have locally"
                        );
                        return Err(anyhow::anyhow!(
                            "Push rejected: The remote branch has commits that are not in your local branch. Pull first before pushing."
                        ));
                    } else {
                        return Err(e.into());
                    }
                }
            }
        } else {
            info!("No commits to push yet");
        }

        info!("Successfully synced with remote repository");
        Ok(())
    }

    /// Helper function to commit a merge
    fn commit_merge(&self, annotated_commit: &git2::AnnotatedCommit, message: &str) -> Result<()> {
        let head_commit = self.repo.head()?.peel_to_commit()?;
        let foreign_commit = self.repo.find_commit(annotated_commit.id())?;

        // Create the merge commit
        let signature = git2::Signature::now("rutd", "rutd@auto.commit")?;
        let tree_id = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&head_commit, &foreign_commit],
        )?;

        // Clean up the merge state
        self.repo.cleanup_state()?;

        Ok(())
    }
}

/// Credential callback for SSH key authentication
fn credential(
    url: &str,
    username_from_url: Option<&str>,
    allowed_types: CredentialType,
) -> Result<Cred, git2::Error> {
    debug!("Attempting authentication for URL: {}", url);
    debug!("Allowed credential types: {:?}", allowed_types);

    // Try SSH key authentication with multiple possible key locations
    if allowed_types.contains(CredentialType::SSH_KEY)
        || allowed_types.contains(CredentialType::SSH_MEMORY)
    {
        if let Ok(home) = env::var("HOME") {
            // Try different common SSH key file names
            let possible_key_paths = [
                format!("{}/.ssh/id_rsa", home),
                format!("{}/.ssh/id_ed25519", home),
                format!("{}/.ssh/id_ecdsa", home),
                format!("{}/.ssh/id_dsa", home),
                format!("{}/.ssh/github_rsa", home),
            ];

            for key_path in &possible_key_paths {
                if Path::new(key_path).exists() {
                    debug!("Trying SSH key: {}", key_path);
                    let username = username_from_url.unwrap_or("git");
                    debug!("Using username: {}", username);

                    match Cred::ssh_key(username, None, Path::new(key_path), None) {
                        Ok(cred) => return Ok(cred),
                        Err(e) => debug!("Failed to use SSH key {}: {}", key_path, e),
                    }
                }
            }
        }

        // Also try SSH agent if available
        if allowed_types.contains(CredentialType::SSH_KEY) {
            debug!("Trying SSH agent authentication");
            if let Ok(cred) = Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
                return Ok(cred);
            }
        }
    }

    // Try username/password if SSH doesn't work
    if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
        debug!("Trying username/password authentication");
        // Check for environment variables containing credentials
        let username = env::var("RUTD_GIT_USERNAME")
            .or_else(|_| env::var("GIT_USERNAME"))
            .unwrap_or_else(|_| username_from_url.unwrap_or("git").to_string());

        if let Ok(password) = env::var("RUTD_GIT_PASSWORD").or_else(|_| env::var("GIT_PASSWORD")) {
            debug!("Using username/password from environment variables");
            return Cred::userpass_plaintext(&username, &password);
        }
    }

    // Fall back to default credentials as last resort
    debug!("Using default credentials (may fail if authentication is required)");
    Cred::default()
}
