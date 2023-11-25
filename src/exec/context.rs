use std::{rc::Rc, any::Any};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::exec::{ActualArguments, ExecError, ExecResult, SourceLocation, Value};

#[derive(Clone)]
pub struct Context
{
    frame: Rc<RefCell<Frame>>,
}

impl Context
{
    pub fn new() -> Context
    {
        let mut result = Context
        {
            frame: Rc::new(RefCell::new(Frame::new()))
        };

        crate::exec::inbuilt::add_inbuilt_functions(&mut result);

        result
    }

    pub fn new_with_state<AppState>(app_state: AppState) -> Context
    where
        AppState: Any
    {
        let mut result = Context
        {
            frame: Rc::new(RefCell::new(Frame::new_with_state(app_state)))
        };

        crate::exec::inbuilt::add_inbuilt_functions(&mut result);

        result
    }

    pub fn get_call_site(&self) -> SourceLocation
    {
        self.frame.borrow().call_site
    }

    pub fn get_var_named(&mut self, source: SourceLocation, name: &str) -> ExecResult<Value>
    {
        match self.frame.borrow().get_var_named(name)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(source, format!("Undefined variable \"{}\"", name))),
        }
    }

    pub fn set_var_named(&mut self, name: &str, value: Value)
    {
        self.frame.borrow_mut().set_var_named(name, value);
    }

    pub fn get_param(&mut self, position: usize, name: &str) -> ExecResult<Value>
    {
        match self.frame.borrow().get_opt_param(position, name)
        {
            Some(val) => Ok(val),
            None => Err(ExecError::new(self.frame.borrow().call_site, format!("Missing required parameter #{}/{}", position, name))),
        }
    }

    pub fn get_opt_param(&mut self, position: usize, name: &str) -> Option<Value>
    {
        self.frame.borrow().get_opt_param(position, name)
    }

    pub fn get_param_all_positional(&mut self) -> Vec<Value>
    {
        self.frame.borrow().get_param_all_positional()
    }

    pub fn sub_frame(&self, call_site: SourceLocation, formal_arguments: &Vec<String>, actual_arguments: ActualArguments) -> Context
    {
        Context
        {
            frame: Rc::new(RefCell::new(Frame::new_frame(Some(self.frame.clone()), call_site, formal_arguments, actual_arguments))),
        }
    }

    pub fn sub_block(&self) -> Context
    {
        Context
        {
            frame: Rc::new(RefCell::new(Frame::new_block(self.frame.clone()))),
        }
    }

    pub fn with_app_state<AppState, Func, Value>(&self, func: Func) -> Result<Value, ExecError>
    where
        AppState: Any,
        Func: FnOnce(&mut AppState) -> Result<Value, ExecError>
    {
        self.frame.borrow().with_app_state::<AppState, Func, Value>(func)
    }

}

struct Frame
{
    parent: Option<Rc<RefCell<Frame>>>,
    call_site: SourceLocation,
    args: ActualArguments,
    vars: HashMap<String, Value>,
    app_state: Option<Rc<RefCell<dyn Any>>>,
}

impl Frame
{
    fn new() -> Self
    {
        Frame
        {
            parent: None,
            call_site: SourceLocation::inbuilt(),
            args: ActualArguments::Positional(Vec::new()),
            vars: HashMap::new(),
            app_state: None,
        }
    }

    fn new_with_state<AppState>(app_state: AppState) -> Self
    where
        AppState: Any
    {
        Frame
        {
            parent: None,
            call_site: SourceLocation::inbuilt(),
            args: ActualArguments::Positional(Vec::new()),
            vars: HashMap::new(),
            app_state: Some(Rc::new(RefCell::new(app_state))),
        }
    }

    fn new_block(parent: Rc<RefCell<Frame>>) -> Self
    {
        let call_site = parent.borrow().call_site;

        Frame
        {
            parent: Some(parent.clone()),
            call_site,
            args: ActualArguments::Positional(Vec::new()),
            vars: HashMap::new(),
            app_state: None,
        }
    }

    fn new_frame(parent: Option<Rc<RefCell<Frame>>>, call_site: SourceLocation, _formal_arguments: &Vec<String>, actual_arguments: ActualArguments) -> Self
    {
        Frame
        {
            parent,
            call_site,
            args: actual_arguments,
            vars: HashMap::new(),
            app_state: None,
        }
    }

    fn get_var_named(&self, name: &str) -> Option<Value>
    {
        if let Some(here) = self.vars.get(name)
        {
            return Some((*here).clone())
        }

        if let ActualArguments::Named(args) = &self.args
        {
            if let Some(val) = args.get(name)
            {
                return Some(val.clone())
            }
        }

        if let Some(parent) = &self.parent
        {
            return parent.borrow().get_var_named(name);
        }

        None
    }

    pub fn set_var_named(&mut self, name: &str, value: Value)
    {
        self.vars.insert(name.to_owned(), value);
    }

    fn get_opt_param(&self, position: usize, name: &str) -> Option<Value>
    {
        match &self.args
        {
            ActualArguments::Named(hash_map) =>
            {
                hash_map.get(name).cloned()
            },
            ActualArguments::Positional(vec) =>
            {
                if position < vec.len()
                {
                    Some(vec[position].clone())
                }
                else
                {
                    None
                }
            },
        }
    }

    fn get_param_all_positional(&self) -> Vec<Value>
    {
        match &self.args
        {
            ActualArguments::Named(_hash_map) =>
            {
                todo!()
            },
            ActualArguments::Positional(vec) =>
            {
                vec.clone()
            },
        }
    }

    fn with_app_state<AppState, Func, Value>(&self, func: Func) -> Result<Value, ExecError>
    where
        AppState: Any,
        Func: FnOnce(&mut AppState) -> Result<Value, ExecError>
    {
        // Try and run ourselves

        if let Some(app_state) = &self.app_state
        {
            if let Some(app_state) = app_state.borrow_mut().downcast_mut::<AppState>()
            {
                return func(app_state);
            }
        }

        // Try and delgate to our parent state

        if let Some(parent) = &self.parent
        {
            return parent.borrow_mut().with_app_state(func);
        }

        // No-one has this state

        Err(ExecError::new(
            self.call_site,
            format!("No app_state for type {}", std::any::type_name::<AppState>())))
    }
}
