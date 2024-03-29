use std::path::PathBuf;

pub mod gltf;
pub mod image;
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

    pub fn path_to_filename(&self, path: &str) -> String
    {
        PathBuf::from(path).file_name().map(|s| s.to_string_lossy()).map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn load_text_file(&self, filename: &str) -> Result<(String, FileSystemContext), ImportError>
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

    pub fn load_binary_file(&self, filename: &str) -> Result<(Vec<u8>, FileSystemContext), ImportError>
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

        match std::fs::read(&filename)
        {
            Ok(contents) => Ok((contents, FileSystemContext{ cwd: combined })),
            Err(err) => Err(ImportError(format!("File System Error: {:?}", err))),
        }
    }
}
