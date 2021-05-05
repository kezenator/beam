use crate::color::LinearRGB;
use crate::desc::CameraDescription;
use crate::exec::{ExecError, ExecResult, Function, SourceLocation};
use crate::geom::Surface;
use crate::geom::sdf::ConcreteSdf;
use crate::material::Material;
use crate::math::Scalar;
use crate::object::Object;
use crate::texture::Texture;
use crate::vec::Vec3;

#[derive(Clone)]
pub enum ValueData
{
    Scalar(Scalar),
    Vec3(Vec3),
    Function(Function),
    Camera(CameraDescription),
    Surface(Box<dyn Surface>),
    Material(Material),
    Color(LinearRGB),
    Object(Object),
    Texture(Texture),
    Sdf(ConcreteSdf),
}

#[derive(Clone)]
pub struct Value
{
    source: SourceLocation,
    data: ValueData,
}

impl Value
{
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

    pub fn new_camera(source: SourceLocation, camera: CameraDescription) -> Value
    {
        Value { source, data: ValueData::Camera(camera) }
    }

    pub fn new_color(source: SourceLocation, color: LinearRGB) -> Value
    {
        Value { source, data: ValueData::Color(color) }
    }

    pub fn new_texture(source: SourceLocation, texture: Texture) -> Value
    {
        Value { source, data: ValueData::Texture(texture) }
    }

    pub fn new_sdf(source: SourceLocation, sdf: ConcreteSdf) -> Value
    {
        Value { source, data: ValueData::Sdf(sdf) }
    }

    pub fn new_surface(source: SourceLocation, surface: Box<dyn Surface>) -> Value
    {
        Value { source, data: ValueData::Surface(surface) }
    }

    pub fn new_material(source: SourceLocation, material: Material) -> Value
    {
        Value { source, data: ValueData::Material(material) }
    }

    pub fn new_object(source: SourceLocation, object: Object) -> Value
    {
        Value { source, data: ValueData::Object(object) }
    }

    pub fn source_location(&self) -> SourceLocation
    {
        self.source
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

    pub fn into_surface(self) -> ExecResult<Box<dyn Surface>>
    {
        match self.data
        {
            ValueData::Surface(val) => Ok(val),
            _ => Err(self.type_error("Surface")),
        }
    }

    pub fn into_material(self) -> ExecResult<Material>
    {
        match self.data
        {
            ValueData::Material(val) => Ok(val),
            _ => Err(self.type_error("Material")),
        }
    }

    pub fn into_color(self) -> ExecResult<LinearRGB>
    {
        match self.data
        {
            ValueData::Color(val) => Ok(val),
            _ => Err(self.type_error("Color")),
        }
    }

    pub fn into_texture(self) -> ExecResult<Texture>
    {
        match self.data
        {
            ValueData::Texture(val) => Ok(val),
            ValueData::Color(val) => Ok(Texture::solid(val)),
            _ => Err(self.type_error("Texture")),
        }
    }

    pub fn into_sdf(self) -> ExecResult<ConcreteSdf>
    {
        match self.data
        {
            ValueData::Sdf(val) => Ok(val),
            _ => Err(self.type_error("Sdf")),
        }
    }

    pub fn into_camera(self) -> ExecResult<CameraDescription>
    {
        match self.data
        {
            ValueData::Camera(val) => Ok(val),
            _ => Err(self.type_error("Camera")),
        }
    }

    pub fn into_object(self) -> ExecResult<Object>
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
