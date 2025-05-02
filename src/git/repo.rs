use std::path::Path;

use anyhow::{Context, Result};
use git2::{Cred, FetchOptions, ObjectType, PushOptions, RemoteCallbacks, Repository, Signature};
use log::{debug, info, warn};

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

    /// Automatically commit changes
    pub fn commit_changes(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
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
    pub fn sync(&self) -> Result<()> {
        // Check if we have any remotes
        let remotes = self.repo.remotes()?;
        if remotes.is_empty() {
            info!("No remote repository configured. Skipping sync.");
            return Ok(());
        }

        info!("Syncing with remote repository...");

        // Set up authentication callbacks
        let credential = |url: &str,
                          username_from_url: Option<&str>,
                          allowed_types: git2::CredentialType| {
            debug!("Attempting authentication for URL: {}", url);
            debug!("Allowed credential types: {:?}", allowed_types);

            // Try SSH key authentication with multiple possible key locations
            if allowed_types.contains(git2::CredentialType::SSH_KEY)
                || allowed_types.contains(git2::CredentialType::SSH_MEMORY)
            {
                if let Ok(home) = std::env::var("HOME") {
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
                if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                    debug!("Trying SSH agent authentication");
                    if let Ok(cred) = Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
                        return Ok(cred);
                    }
                }
            }

            // Try username/password if SSH doesn't work
            if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                debug!("Trying username/password authentication");
                // Check for environment variables containing credentials
                let username = std::env::var("RUTD_GIT_USERNAME")
                    .or_else(|_| std::env::var("GIT_USERNAME"))
                    .unwrap_or_else(|_| username_from_url.unwrap_or("git").to_string());

                if let Ok(password) =
                    std::env::var("RUTD_GIT_PASSWORD").or_else(|_| std::env::var("GIT_PASSWORD"))
                {
                    debug!("Using username/password from environment variables");
                    return Cred::userpass_plaintext(&username, &password);
                }
            }

            // Fall back to default credentials as last resort
            debug!("Using default credentials (may fail if authentication is required)");
            Cred::default()
        };
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
                if e.code() == git2::ErrorCode::UnbornBranch
                    || e.code() == git2::ErrorCode::NotFound
                {
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

            // Create merge analysis
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
                    .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;

                info!("Successfully pulled changes from remote");
            } else {
                info!("Merge required, but automatic merging is not supported yet");
                // Future enhancement: implement proper merge conflict
                // resolution
            }
        } else {
            warn!(
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
}
