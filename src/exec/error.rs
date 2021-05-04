use crate::exec::SourceLocation;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecError
{
    source: SourceLocation,
    msg: String,
}

impl ExecError
{
    pub fn new<S: Into<String>>(location: SourceLocation, msg: S) -> Self
    {
        ExecError
        {
            source: location,
            msg: msg.into(),
        }
    }

    pub fn new_no_loc<S: Into<String>>(msg: S) -> Self
    {
        ExecError
        {
            source: SourceLocation::inbuilt(),
            msg: msg.into(),
        }
    }

    pub fn message(&self) -> String
    {
        self.msg.clone()
    }
}
