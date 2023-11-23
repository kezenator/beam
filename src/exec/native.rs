use crate::exec::{Context, ExecResult, FromValue, Function, Value};

pub struct NativeFunctionBuilder
{
    context: Context,
    funcs: Vec<Function>,
}

impl NativeFunctionBuilder
{
    pub fn new(context: &mut Context) -> Self
    {
        NativeFunctionBuilder
        {
            context: context.clone(),
            funcs: Vec::new(),
        }
    }

    pub fn build(self) -> Vec<Function>
    {
        self.funcs
    }

    pub fn add_vec<N, F, T1>(&mut self, names: N, arg: &str, func: F)
        where N: IntoFunctionNameSet,
            F: Fn(&mut Context, Vec<T1>) -> ExecResult<Value> + Copy + 'static,
            T1: FromValue
    {
        for name in names.into_names()
        {
            self.funcs.push(Function::new_inbuilt(
                name,
                vec![arg.to_string()],
                &mut self.context.clone(),
                move |context|
                {
                    let vec = context.get_param_all_positional()
                        .into_iter()
                        .map(|v| T1::from_value(v, context))
                        .collect::<ExecResult<Vec<T1>>>()?;
                    func(context, vec)
                }));
        }
    }

    pub fn add_0<N, F>(&mut self, names: N, func: F)
        where N: IntoFunctionNameSet,
            F: Fn(&mut Context) -> ExecResult<Value> + Copy + 'static
    {
        for name in names.into_names()
        {
            self.funcs.push(Function::new_inbuilt(
                name,
                Vec::new(),
                &mut self.context.clone(),
                move |context|
                {
                    func(context)
                }));
        }
    }

    pub fn add_1<N, F, T1>(&mut self, names: N, args: [&str;1], func: F)
        where N: IntoFunctionNameSet,
            F: Fn(&mut Context, T1) -> ExecResult<Value> + Copy + 'static,
            T1: FromValue
    {
        for name in names.into_names()
        {
            self.funcs.push(Function::new_inbuilt(
                name.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
                &mut self.context.clone(),
                move |context|
                {
                    let v1 = T1::from_value(context.get_param_positional(0)?, context)?;
                    func(context, v1)
                }));
        }
    }

    pub fn add_2<N, F, T1, T2>(&mut self, names: N, args: [&str;2], func: F)
        where N: IntoFunctionNameSet,
            F: Fn(&mut Context, T1, T2) -> ExecResult<Value> + Copy + 'static,
            T1: FromValue,
            T2: FromValue,
    {
        for name in names.into_names()
        {
            self.funcs.push(Function::new_inbuilt(
                name.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
                &mut self.context.clone(),
                move |context|
                {
                    let v1 = T1::from_value(context.get_param_positional(0)?, context)?;
                    let v2 = T2::from_value(context.get_param_positional(1)?, context)?;
                    func(context, v1, v2)
                }));
        }
    }

    pub fn add_3<N, F, T1, T2, T3>(&mut self, names: N, args: [&str;3], func: F)
        where N: IntoFunctionNameSet,
            F: Fn(&mut Context, T1, T2, T3) -> ExecResult<Value> + Copy + 'static,
            T1: FromValue,
            T2: FromValue,
            T3: FromValue,
    {
        for name in names.into_names()
        {
            self.funcs.push(Function::new_inbuilt(
                name.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
                &mut self.context.clone(),
                move |context|
                {
                    let v1 = T1::from_value(context.get_param_positional(0)?, context)?;
                    let v2 = T2::from_value(context.get_param_positional(1)?, context)?;
                    let v3 = T3::from_value(context.get_param_positional(2)?, context)?;
                    func(context, v1, v2, v3)
                }));
        }
    }

    pub fn add_4<N, F, T1, T2, T3, T4>(&mut self, names: N, args: [&str;4], func: F)
        where N: IntoFunctionNameSet,
            F: Fn(&mut Context, T1, T2, T3, T4) -> ExecResult<Value> + Copy + 'static,
            T1: FromValue,
            T2: FromValue,
            T3: FromValue,
            T4: FromValue,
    {
        for name in names.into_names()
        {
            self.funcs.push(Function::new_inbuilt(
                name.to_string(),
                args.iter().map(|a| a.to_string()).collect(),
                &mut self.context.clone(),
                move |context|
                {
                    let v1 = T1::from_value(context.get_param_positional(0)?, context)?;
                    let v2 = T2::from_value(context.get_param_positional(1)?, context)?;
                    let v3 = T3::from_value(context.get_param_positional(2)?, context)?;
                    let v4 = T4::from_value(context.get_param_positional(3)?, context)?;
                    func(context, v1, v2, v3, v4)
                }));
        }
    }
}

pub trait IntoFunctionNameSet
{
    fn into_names(&self) -> Vec<String>;
}

impl IntoFunctionNameSet for &str
{
    fn into_names(&self) -> Vec<String>
    {
        vec![self.to_string()]
    }
}

impl<const N: usize> IntoFunctionNameSet for [&str;N]
{
    fn into_names(&self) -> Vec<String>
    {
        self.iter().map(|s| s.to_string()).collect()
    }
}
