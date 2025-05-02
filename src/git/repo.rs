use std::path::Path;

use anyhow::Result;
use git2::{Commit, ObjectType, Repository, Signature};

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
        match head {
            Some(ref head_commit) => {
                self.repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[head_commit],
                )?;
            }
            None => {
                self.repo
                    .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;
            }
        }

        Ok(())
    }

    /// Generate commit message
    pub fn generate_commit_message(task_id: &str, action: &str) -> String {
        format!("chore({}): {}", task_id, action)
    }
}
