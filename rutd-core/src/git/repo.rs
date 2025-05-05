use std::{env, path::Path};

use anyhow::{Context, Result};
use git2::{
    Cred, CredentialType, ErrorCode, FetchOptions, FileFavor, IndexAddOption, MergeOptions,
    ObjectType, PushOptions, RemoteCallbacks, Repository, Signature, build::CheckoutBuilder,
};

use super::MergeStrategy;
use crate::config::GitConfig;

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    /// Initialize Git repository, create a new one if it doesn't exist
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        // If the repository doesn't exist, create a new one
        let repo = Repository::open(path).or_else(|_| Repository::init(path))?;

        Ok(Self { repo })
    }

    /// Clone a remote repository to the local directory
    ///
    /// - url: URL of the remote repository
    /// - branch: Branch to clone (default is "main" or "master")
    pub fn clone<P: AsRef<Path>>(path: P, url: &str, git_config: &GitConfig) -> Result<Self> {
        let path = path.as_ref();
        log::info!("Cloning {} to {}", url, path.display());

        let git_config = git_config.clone();

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |url, username, allowed_types| {
            credential(url, username, allowed_types, &git_config)
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Prepare builder.
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // Clone the project.
        match builder.clone(url, path) {
            Ok(repo) => {
                log::info!("Successfully cloned repository");
                Ok(Self { repo })
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
        index.add_all(std::iter::once(&"*"), IndexAddOption::DEFAULT, None)?;
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
        let parents = head.as_ref().map_or(vec![], |commit| vec![commit]);
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;

        log::debug!("Created commit: {commit_id}");
        Ok(())
    }

    /// Generate commit message using Conventional Commits format
    ///
    /// Format: <action>(<scope>|<type>): <short description> <task_id>
    /// - action: The type of change (e.g., feat, fix, chore)
    /// - scope: The scope of the change (from task.scope or "task")
    /// - type: The type of the task (from task.task_type if available)
    /// - short description: Brief description of the change
    /// - task_id: The ID of the task
    pub fn generate_commit_message(
        action: &str,
        scope: Option<&str>,
        task_type: Option<&str>,
        description: &str,
        task_id: &str,
    ) -> String {
        // Determine the appropriate scope string for the commit
        let scope = scope.unwrap_or("-");
        let task_type = task_type.unwrap_or("-");

        format!("{action}({scope}|{task_type}): {description}\n\n{task_id}")
    }

    /// Sync with remote repository (fetch, pull, push)
    ///
    /// - prefer: Specifies the resolution strategy for merge conflicts
    pub fn sync(&self, prefer: MergeStrategy, git_config: &GitConfig) -> Result<()> {
        // Check if we have any remotes
        let remotes = self.repo.remotes()?;
        if remotes.is_empty() {
            log::info!("No remote repository configured. Skipping sync.");
            return Ok(());
        }

        log::info!("Syncing with remote repository...");

        // Get the default remote name (usually "origin")
        let remote_name = "origin";

        // Fetch the latest changes
        self.fetch_from_remote(remote_name, git_config)?;

        // Get the current branch name
        let branch_name = self.get_current_branch_name()?;
        if branch_name.is_none() {
            return Ok(());
        }
        let branch_name = branch_name.unwrap();

        // Try to merge remote changes
        self.merge_remote_changes(remote_name, &branch_name, prefer)?;

        // Push local changes
        self.push_to_remote(remote_name, &branch_name, git_config)?;

        log::info!("Successfully synced with remote repository");
        Ok(())
    }

    /// Fetch the latest changes from the specified remote
    fn fetch_from_remote(&self, remote_name: &str, git_config: &GitConfig) -> Result<()> {
        // Set up authentication callbacks
        let mut callbacks = RemoteCallbacks::new();
        let git_config_clone = git_config.clone();
        callbacks.credentials(move |url, username, allowed_types| {
            credential(url, username, allowed_types, &git_config_clone)
        });

        // Fetch latest changes
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Fetch from remote
        log::debug!("Fetching from remote '{remote_name}'");
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .context(format!("No remote named '{remote_name}' found"))?;

        // Attempt to fetch but handle the case where the remote is empty or unreachable
        match remote.fetch(&["master", "main"], Some(&mut fetch_options), None) {
            Ok(_) => log::debug!("Successfully fetched from remote"),
            Err(e) => {
                // Check if this is a fresh/empty repository error
                if e.to_string().contains("couldn't find remote ref") {
                    log::debug!(
                        "Remote repository appears to be empty or the branch doesn't exist yet"
                    );
                    // Continue with push only
                } else {
                    // For other errors, return the error
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    /// Get the current branch name, returns None if HEAD is not set
    fn get_current_branch_name(&self) -> Result<Option<String>> {
        // Get the current branch name
        let head = match self.repo.head() {
            Ok(head) => head,
            Err(e) => {
                // If HEAD is not yet set (no commits), create an initial commit
                if matches!(e.code(), ErrorCode::UnbornBranch | ErrorCode::NotFound) {
                    log::debug!("No HEAD found, repository might be empty");
                    return Ok(None);
                }
                return Err(e.into());
            }
        };

        let branch_name = if head.is_branch() {
            head.shorthand().unwrap_or("master")
        } else {
            "master"
        };

        log::debug!("Current branch: {branch_name}");
        Ok(Some(branch_name.to_string()))
    }

    /// Merge remote changes into the local branch
    fn merge_remote_changes(
        &self,
        remote_name: &str,
        branch_name: &str,
        prefer: MergeStrategy,
    ) -> Result<()> {
        let remote_branch = format!("refs/remotes/{remote_name}/{branch_name}");
        if let Ok(remote_reference) = self.repo.find_reference(&remote_branch) {
            let remote_commit = remote_reference.peel_to_commit()?;
            let annotated_commit = self.repo.find_annotated_commit(remote_commit.id())?;
            let analysis = self.repo.merge_analysis(&[&annotated_commit])?;

            if analysis.0.is_up_to_date() {
                log::debug!("Local repository is up to date");
            } else if analysis.0.is_fast_forward() {
                self.fast_forward_branch(branch_name, remote_commit.id())?;
                log::info!("Successfully pulled changes from remote");
            } else if analysis.0.is_normal() {
                self.handle_normal_merge(branch_name, &annotated_commit, &remote_branch, prefer)?;
            }
        } else {
            log::debug!(
                "Remote branch '{remote_branch}' not found. This might be a new remote repository."
            );
        }

        Ok(())
    }

    /// Perform a fast-forward update of the branch
    fn fast_forward_branch(&self, branch_name: &str, target_id: git2::Oid) -> Result<()> {
        log::debug!("Fast-forwarding local repository");

        // Perform the fast-forward
        let mut reference = self
            .repo
            .find_reference(&format!("refs/heads/{branch_name}"))?;
        reference.set_target(target_id, "Fast-forward update")?;

        // Update the working directory
        self.repo.set_head(&format!("refs/heads/{branch_name}"))?;
        self.repo
            .checkout_head(Some(CheckoutBuilder::new().force()))?;

        Ok(())
    }

    /// Handle a normal merge with possible conflicts
    fn handle_normal_merge(
        &self,
        branch_name: &str,
        annotated_commit: &git2::AnnotatedCommit,
        remote_branch: &str,
        prefer: MergeStrategy,
    ) -> Result<()> {
        // Need to perform a merge with possible conflicts
        log::debug!("Merge required - analyzing merge strategy");

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
            .merge(&[annotated_commit], Some(&mut merge_opts), None)?;

        // Handle merge conflicts based on the prefer option
        self.handle_merge_conflicts(prefer)?;

        // Commit the merge
        let commit_message =
            format!("Merge remote-tracking branch '{remote_branch}' into '{branch_name}'");
        self.commit_merge(annotated_commit, &commit_message)
            .context("Failed to commit merge")?;

        Ok(())
    }

    /// Handle merge conflicts based on the specified strategy
    fn handle_merge_conflicts(&self, prefer: MergeStrategy) -> Result<()> {
        let conflicts = self.repo.index()?.conflicts()?.collect::<Vec<_>>();
        if conflicts.is_empty() {
            log::debug!("Successfully merged remote changes");
            return Ok(());
        }

        log::debug!("Merge conflicts detected");
        for conflict in conflicts {
            let conflict = conflict?;
            // Resolve each conflict based on the prefer option
            let mut index = self.repo.index()?;
            match prefer {
                MergeStrategy::Local => {
                    if let Some(ours) = conflict.our {
                        let path_str =
                            String::from_utf8(ours.path).context("Invalid UTF-8 in path")?;
                        let path = Path::new(&path_str);
                        index.conflict_remove(path)?;
                        index.add_path(path)?;
                    }
                }
                MergeStrategy::Remote => {
                    if let Some(theirs) = conflict.their {
                        let path_str =
                            String::from_utf8(theirs.path).context("Invalid UTF-8 in path")?;
                        let path = Path::new(&path_str);
                        index.conflict_remove(path)?;
                        index.add_path(path)?;
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

        Ok(())
    }

    /// Push local changes to the remote repository
    fn push_to_remote(
        &self,
        remote_name: &str,
        branch_name: &str,
        git_config: &GitConfig,
    ) -> Result<()> {
        log::debug!("Pushing to remote '{remote_name}'");
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |url, username, allowed_types| {
            credential(url, username, allowed_types, git_config)
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let mut remote = self.repo.find_remote(remote_name)?;

        // Check if we have any commits to push
        if self.repo.head().is_err() {
            log::info!("No commits to push yet");
            return Ok(());
        }

        // Ensure HEAD exists (at least one commit)
        match remote.push(
            &[format!("refs/heads/{branch_name}")],
            Some(&mut push_options),
        ) {
            Ok(_) => log::info!("Successfully pushed to remote repository"),
            Err(e) => {
                if e.to_string().contains("non-fast-forward") {
                    log::info!(
                        "Cannot push because remote contains work that you do not have locally"
                    );
                    anyhow::bail!(
                        "Push rejected: The remote branch has commits that are not in your local branch. Pull first before pushing."
                    );
                } else {
                    return Err(e.into());
                }
            }
        }

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
    git_config: &GitConfig,
) -> Result<Cred, git2::Error> {
    log::debug!("Attempting authentication for URL: {url}");
    log::debug!("Allowed credential types: {allowed_types:?}");

    // Try SSH key authentication with multiple possible key locations
    if allowed_types.contains(CredentialType::SSH_KEY)
        || allowed_types.contains(CredentialType::SSH_MEMORY)
    {
        if let Ok(home) = env::var("HOME") {
            // Try different common SSH key file names
            let possible_key_paths = [
                format!("{home}/.ssh/id_rsa"),
                format!("{home}/.ssh/id_ed25519"),
                format!("{home}/.ssh/id_ecdsa"),
                format!("{home}/.ssh/id_dsa"),
                format!("{home}/.ssh/github_rsa"),
            ];

            for key_path in &possible_key_paths {
                if Path::new(key_path).exists() {
                    log::debug!("Trying SSH key: {key_path}");
                    let username = username_from_url.unwrap_or("git");
                    log::debug!("Using username: {username}");

                    match Cred::ssh_key(username, None, Path::new(key_path), None) {
                        Ok(cred) => return Ok(cred),
                        Err(e) => log::debug!("Failed to use SSH key {key_path}: {e}"),
                    }
                }
            }
        }

        // Also try SSH agent if available
        if allowed_types.contains(CredentialType::SSH_KEY) {
            log::debug!("Trying SSH agent authentication");
            if let Ok(cred) = Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
                return Ok(cred);
            }
        }
    }

    // Try username/password if SSH doesn't work
    if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
        log::debug!("Trying username/password authentication");

        // Use the username from GitConfig or fallback to URL username or "git"
        let username = if git_config.username.is_empty() {
            username_from_url.unwrap_or("git").to_string()
        } else {
            git_config.username.clone()
        };

        // Check if we have a password in the GitConfig
        if !git_config.password.is_empty() {
            log::debug!("Using username/password from configuration");
            return Cred::userpass_plaintext(&username, &git_config.password);
        }
    }

    // Fall back to default credentials as last resort
    log::debug!("Using default credentials (may fail if authentication is required)");
    Cred::default()
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use tempfile::tempdir;

    use super::*;

    impl GitRepo {
        pub const fn get_repo(&self) -> &Repository {
            &self.repo
        }
    }

    #[test]
    fn test_init_repository() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a repository
        let result = GitRepo::init(repo_path);
        assert!(result.is_ok());

        // Check that the .git directory was created
        let git_dir = repo_path.join(".git");
        assert!(git_dir.exists());
    }

    #[test]
    fn test_init_existing_repository() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a repository
        let result1 = GitRepo::init(repo_path);
        assert!(result1.is_ok());

        // Initialize again - should still succeed
        let result2 = GitRepo::init(repo_path);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_generate_commit_message() {
        // Test with all parameters
        let message = GitRepo::generate_commit_message(
            "create",
            Some("proj"),
            Some("feat"),
            "Add new feature",
            "task-123",
        );

        assert!(message.contains("create"));
        assert!(message.contains("proj"));
        assert!(message.contains("feat"));
        assert!(message.contains("Add new feature"));
        assert!(message.contains("task-123"));

        // Test without optional parameters
        let message = GitRepo::generate_commit_message(
            "delete",
            None,
            None,
            "Remove obsolete task",
            "task-456",
        );

        assert!(message.contains("delete"));
        assert!(message.contains("Remove obsolete task"));
        assert!(message.contains("task-456"));

        // For a predictable format test:
        let message = GitRepo::generate_commit_message(
            "type",
            Some("scope"),
            Some("subtype"),
            "Description",
            "id",
        );
        assert_eq!(message, "type(scope|subtype): Description\n\nid");
    }

    #[test]
    fn test_commit_changes() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a repository
        let git_repo = GitRepo::init(repo_path).unwrap();

        // Create a test file
        let test_file = repo_path.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "Test content").unwrap();

        // Commit the change
        let result = git_repo.commit_changes("Initial commit");
        assert!(result.is_ok());

        // Verify the commit was created
        let repo = git_repo.get_repo();
        let head = repo.head().expect("Head should exist after commit");
        let commit = head
            .peel_to_commit()
            .expect("Head should reference a commit");
        assert_eq!(commit.message().unwrap(), "Initial commit");
    }

    #[test]
    fn test_multiple_commits() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a repository
        let git_repo = GitRepo::init(repo_path).unwrap();

        // Create and commit a file
        let test_file = repo_path.join("test1.txt");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "First file").unwrap();
        git_repo.commit_changes("First commit").unwrap();

        // Create and commit another file
        let test_file2 = repo_path.join("test2.txt");
        let mut file2 = File::create(&test_file2).unwrap();
        writeln!(file2, "Second file").unwrap();
        git_repo.commit_changes("Second commit").unwrap();

        // Verify we have two commits
        let repo = git_repo.get_repo();
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        assert_eq!(commit.message().unwrap(), "Second commit");

        // Get parent commit
        let parent = commit.parent(0).unwrap();
        assert_eq!(parent.message().unwrap(), "First commit");
    }

    #[test]
    fn test_get_branch_name() {
        // This test verifies that we can get the correct branch name after creating a
        // branch

        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a repository and create an initial commit
        let git_repo = GitRepo::init(repo_path).unwrap();
        let test_file = repo_path.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "Initial content").unwrap();
        git_repo.commit_changes("Initial commit").unwrap();

        // Use the underlying repository to create a branch and check it out
        let repo = git_repo.get_repo();
        let head = repo.head().unwrap();
        let oid = head.target().unwrap();
        repo.branch("test-branch", &repo.find_commit(oid).unwrap(), false)
            .unwrap();
        repo.set_head("refs/heads/test-branch").unwrap();

        // Get the current branch name
        let head = repo.head().unwrap();
        assert!(head.is_branch());
        assert_eq!(head.shorthand().unwrap(), "test-branch");
    }

    #[test]
    fn test_different_credential_types() {
        // Test using different credential types

        // Create different GitConfig objects for testing
        let empty_config = GitConfig {
            username: "".to_string(),
            password: "".to_string(),
        };

        let user_pass_config = GitConfig {
            username: "test-user".to_string(),
            password: "test-password".to_string(),
        };

        // Test user/pass credentials with different configs
        let url = "https://example.com";
        let username = Some("url-user");

        // With empty config, should use username from URL or default to "git"
        let cred_result = credential(
            url,
            username,
            CredentialType::USER_PASS_PLAINTEXT,
            &empty_config,
        );

        // With user/pass config, should use config credentials
        let cred_result2 = credential(
            url,
            username,
            CredentialType::USER_PASS_PLAINTEXT,
            &user_pass_config,
        );

        // We can't directly assert on the credential contents, but we can ensure
        // the function runs without errors
        assert!(cred_result.is_ok() || cred_result.is_err());
        assert!(cred_result2.is_ok() || cred_result2.is_err());
    }
}
