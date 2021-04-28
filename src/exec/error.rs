use crate::exec::SourceLocation;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecError
{
    msg: String,
}

impl ExecError
{
    pub fn new<S: Into<String>>(location: SourceLocation, msg: S) -> Self
    {
        let _ = location;

        Self::new_no_loc(msg)
    }

    pub fn new_no_loc<S: Into<String>>(msg: S) -> Self
    {
        ExecError
        {
            msg: msg.into(),
        }
    }

    pub fn message(&self) -> String
    {
        self.msg.clone()
    }
}
