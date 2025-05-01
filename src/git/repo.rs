use git2::{Repository, Signature, Commit, ObjectType};
use std::path::Path;
use std::error::Error;

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    /// Initialize Git repository, create a new one if it doesn't exist
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        let repo = match Repository::open(path) {
            Ok(repo) => repo,
            Err(_) => {
                // 如果仓库不存在，则初始化一个新的
                Repository::init(path)?
            }
        };
        Ok(GitRepo { repo })
    }

    /// Automatically commit changes
    pub fn commit_changes(&self, message: &str) -> Result<(), Box<dyn Error>> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let signature = Signature::now("rutd", "rutd@example.com")?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        // 获取当前 HEAD 提交，如果存在
        let head = match self.repo.head() {
            Ok(head) => Some(head.peel(ObjectType::Commit)?.into_commit().unwrap()),
            Err(_) => None,
        };

        // 创建提交
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
                self.repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[],
                )?;
            }
        }

        Ok(())
    }

    /// Generate commit message
    pub fn generate_commit_message(task_id: &str, action: &str) -> String {
        format!("chore({}): {}", task_id, action)
    }
}