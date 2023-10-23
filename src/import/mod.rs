use std::path::PathBuf;

pub mod obj;

#[derive(Debug, Clone)]
pub struct ImportError(pub String);

pub struct FileSystemContext
{
    cwd: PathBuf,
}

impl FileSystemContext
{
    pub fn new() -> Self
    {
        FileSystemContext { cwd: std::env::current_dir().unwrap_or(PathBuf::new()) }
    }

    pub fn load_file(&self, filename: &str) -> Result<(String, FileSystemContext), ImportError>
    {
        if filename.is_empty()
        {
            return Err(ImportError("Empty filename".into()));
        }

        let filename = self.cwd.join(PathBuf::from(filename));
        let file_dir = filename.parent().unwrap().to_owned();
        let combined = self.cwd
            .join(file_dir)
            .canonicalize()
            .map_err(|err| ImportError(format!("File System Error: {:?}", err)))?;

        match std::fs::read_to_string(&filename)
        {
            Ok(contents) => Ok((contents, FileSystemContext{ cwd: combined })),
            Err(err) => Err(ImportError(format!("File System Error: {:?}", err))),
        }
    }
}