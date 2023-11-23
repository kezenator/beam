use crate::desc::edit::{Camera, Color, Scene, Texture};
use crate::geom::Aabb;
use crate::indexed::{MaterialIndex, GeomIndex, ObjectIndex, TextureIndex};
use crate::exec::{Context, ExecError, ExecResult, Function, SourceLocation};
use crate::geom::sdf::Sdf;
use crate::math::Scalar;
use crate::vec::Vec3;

#[derive(Clone)]
pub enum ValueData
{
    Void,
    Bool(bool),
    String(String),
    Scalar(Scalar),
    Vec3(Vec3),
    Function(Function),
    Camera(Camera),
    Geom(GeomIndex),
    Aabb(Aabb),
    Material(MaterialIndex),
    Color(Color),
    Object(ObjectIndex),
    Texture(TextureIndex),
    Sdf(Sdf),
}

#[derive(Clone)]
pub struct Value
{
    source: SourceLocation,
    data: ValueData,
}

impl Value
{
    pub fn new_void() -> Value
    {
        Value { source: SourceLocation::inbuilt(), data: ValueData::Void }
    }

    pub fn new_bool(source: SourceLocation, val: bool) -> Value
    {
        Value { source, data: ValueData::Bool(val) }
    }

    pub fn new_string(source: SourceLocation, val: String) -> Value
    {
        Value { source, data: ValueData::String(val) }
    }

    pub fn new_scalar(source: SourceLocation, val: Scalar) -> Value
    {
        Value { source, data: ValueData::Scalar(val) }
    }

    pub fn new_vec3(source: SourceLocation, val: Vec3) -> Value
    {
        Value { source, data: ValueData::Vec3(val) }
    }

    pub fn new_function(function: Function) -> Value
    {
        Value { source: function.get_source_location(), data: ValueData::Function(function), }
    }

    pub fn new_camera(source: SourceLocation, camera: Camera) -> Value
    {
        Value { source, data: ValueData::Camera(camera) }
    }

    pub fn new_color(source: SourceLocation, color: Color) -> Value
    {
        Value { source, data: ValueData::Color(color) }
    }

    pub fn new_texture(source: SourceLocation, texture: TextureIndex) -> Value
    {
        Value { source, data: ValueData::Texture(texture) }
    }

    pub fn new_sdf(source: SourceLocation, sdf: Sdf) -> Value
    {
        Value { source, data: ValueData::Sdf(sdf) }
    }

    pub fn new_geom(source: SourceLocation, geom: GeomIndex) -> Value
    {
        Value { source, data: ValueData::Geom(geom) }
    }

    pub fn new_aabb(source: SourceLocation, aabb: Aabb) -> Value
    {
        Value { source, data: ValueData::Aabb(aabb) }
    }

    pub fn new_material(source: SourceLocation, material: MaterialIndex) -> Value
    {
        Value { source, data: ValueData::Material(material) }
    }

    pub fn new_object(source: SourceLocation, object: ObjectIndex) -> Value
    {
        Value { source, data: ValueData::Object(object) }
    }

    pub fn source_location(&self) -> SourceLocation
    {
        self.source
    }
    
    pub fn into_bool(self) -> ExecResult<bool>
    {
        match self.data
        {
            ValueData::Bool(val) => Ok(val),
            _ => Err(self.type_error("Bool")),
        }
    }
    
    pub fn into_string(self) -> ExecResult<String>
    {
        match self.data
        {
            ValueData::String(val) => Ok(val),
            _ => Err(self.type_error("String")),
        }
    }
    
    pub fn into_scalar(self) -> ExecResult<Scalar>
    {
        match self.data
        {
            ValueData::Scalar(val) => Ok(val),
            _ => Err(self.type_error("Scalar")),
        }
    }

    pub fn into_vec3(self) -> ExecResult<Vec3>
    {
        match self.data
        {
            ValueData::Vec3(val) => Ok(val),
            _ => Err(self.type_error("Vec3")),
        }
    }

    pub fn into_function(self) -> ExecResult<Function>
    {
        match self.data
        {
            ValueData::Function(func) => Ok(func),
            _ => Err(self.type_error("Function")),
        }
    }

    pub fn into_geom(self) -> ExecResult<GeomIndex>
    {
        match self.data
        {
            ValueData::Geom(val) => Ok(val),
            _ => Err(self.type_error("Geom")),
        }
    }

    pub fn into_aabb(self) -> ExecResult<Aabb>
    {
        match self.data
        {
            ValueData::Aabb(val) => Ok(val),
            _ => Err(self.type_error("Aabb")),
        }
    }

    pub fn into_material(self) -> ExecResult<MaterialIndex>
    {
        match self.data
        {
            ValueData::Material(val) => Ok(val),
            _ => Err(self.type_error("Material")),
        }
    }

    pub fn into_color(self) -> ExecResult<Color>
    {
        match self.data
        {
            ValueData::Color(val) => Ok(val),
            _ => Err(self.type_error("Color")),
        }
    }

    pub fn into_texture(self, context: &mut Context) -> ExecResult<TextureIndex>
    {
        match self.data
        {
            ValueData::Texture(texture) => Ok(texture),
            ValueData::Color(color) => Ok(context.with_app_state::<Scene, _, _>(|scene| Ok(scene.collection.push(Texture::Solid(color))))?),
            _ => Err(self.type_error("Texture")),
        }
    }

    pub fn into_sdf(self) -> ExecResult<Sdf>
    {
        match self.data
        {
            ValueData::Sdf(val) => Ok(val),
            _ => Err(self.type_error("Sdf")),
        }
    }

    pub fn into_camera(self) -> ExecResult<Camera>
    {
        match self.data
        {
            ValueData::Camera(val) => Ok(val),
            _ => Err(self.type_error("Camera")),
        }
    }

    pub fn into_object(self) -> ExecResult<ObjectIndex>
    {
        match self.data
        {
            ValueData::Object(val) => Ok(val),
            _ => Err(self.type_error("Object")),
        }
    }

    fn type_error(&self, expected: &str) -> ExecError
    {
        ExecError::new(self.source, format!("Expected {}", expected))
    }
}

pub trait FromValue
    where Self: Sized
{
    const IS_OPTIONAL: bool;
    fn from_value(value: Value, context: &mut Context) -> ExecResult<Self>;
}

impl FromValue for Value
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<Value>
    {
        Ok(value)
    }
}

impl<T> FromValue for Option<T>
    where T: FromValue
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, context: &mut Context) -> ExecResult<Option<T>>
    {
        Ok(Some(T::from_value(value, context)?))
    }
}

impl FromValue for bool
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<bool>
    {
        value.into_bool()
    }
}

impl FromValue for Scalar
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<Scalar>
    {
        value.into_scalar()
    }
}

impl FromValue for Vec3
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<Vec3>
    {
        value.into_vec3()
    }
}

impl FromValue for Color
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<Color>
    {
        value.into_color()
    }
}

impl FromValue for Aabb
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<Aabb>
    {
        value.into_aabb()
    }
}

impl FromValue for Sdf
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<Sdf>
    {
        value.into_sdf()
    }
}

impl FromValue for TextureIndex
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, context: &mut Context) -> ExecResult<TextureIndex>
    {
        value.into_texture(context)
    }
}

impl FromValue for MaterialIndex
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<MaterialIndex>
    {
        value.into_material()
    }
}

impl FromValue for GeomIndex
{
    const IS_OPTIONAL: bool = false;

    fn from_value(value: Value, _: &mut Context) -> ExecResult<GeomIndex>
    {
        value.into_geom()
    }
}