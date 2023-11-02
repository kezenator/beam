use crate::import::ImportError;
use crate::import::obj::obj_file::{Triangle, Vertex};

pub struct Parser<'a>
{
    context: &'a str,
    lines: Vec<&'a str>,
    cur_line_num: usize,
    cur_line_parts: Vec<&'a str>,
}

impl<'a> Parser<'a>
{
    pub fn new(contents: &'a str, context: &'a str) -> Self
    {
        let lines = contents.lines().collect();

        let mut result = Parser
        {
            context,
            lines,
            cur_line_num: 0,
            cur_line_parts: Vec::new(),
        };

        result.to_next_line();

        result
    }

    pub fn is_empty(&self) -> bool
    {
        self.cur_line_num > self.lines.len()
    }

    pub fn first_part(&self) -> &'a str
    {
        self.cur_line_parts[0]
    }

    pub fn ignore_line(&mut self)
    {
        self.to_next_line();
    }

    pub fn parse_line_1_string(&mut self) -> Result<&'a str, ImportError>
    {
        if self.cur_line_parts.len() != 2
        {
            return Err(self.create_error("Expected 1 string parameter"));
        }
        let result = self.cur_line_parts[1];
        self.to_next_line();
        Ok(result)
    }

    pub fn parse_line_1_float(&mut self) -> Result<f64, ImportError>
    {
        if self.cur_line_parts.len() != 2
        {
            return Err(self.create_error("Expected 1 float parameter"));
        }
        let result = self.cur_line_parts[1].parse::<f64>().map_err(|_| self.create_error("Invalid float parameter"))?;
        self.to_next_line();
        Ok(result)
    }

    pub fn parse_line_vector(&mut self) -> Result<(f64, f64, f64), ImportError>
    {
        if (self.cur_line_parts.len() < 2) || (self.cur_line_parts.len() > 4)
        {
            return Err(self.create_error("Expected 1, 2 or 3 float parameters"));
        }

        let result1 = self.cur_line_parts[1].parse::<f64>().map_err(|_| self.create_error("Invalid float parameter"))?;
        let mut result2 = 0.0;
        let mut result3 = 0.0;

        if self.cur_line_parts.len() >= 3
        {
            result2 = self.cur_line_parts[2].parse::<f64>().map_err(|_| self.create_error("Invalid float parameter"))?;
        }

        if self.cur_line_parts.len() >= 4
        {
            result3 = self.cur_line_parts[3].parse::<f64>().map_err(|_| self.create_error("Invalid float parameter"))?;
        }

        self.to_next_line();

        Ok((result1, result2, result3))
    }

    pub fn parse_line_triangles(&mut self, num_verticies: usize, num_texture_coords: usize, num_normals: usize) -> Result<Vec<Triangle>, ImportError>
    {
        if self.cur_line_parts.len() < 3
        {
            return Err(self.create_error("Expected at least 3 vertices"));
        }

        let vertices = self.cur_line_parts[1..].iter().map(|part| self.parse_vertex(part, num_verticies, num_texture_coords, num_normals)).collect::<Vec<_>>();

        for vertex in vertices.iter()
        {
            if let Err(err) = vertex
            {
                return Err(err.clone());
            }
        }

        let mut triangles = Vec::new();
        triangles.reserve(vertices.len() - 2);

        for i in 1..(vertices.len() - 1)
        {
            triangles.push([
                vertices[0].clone().unwrap(),
                vertices[i].clone().unwrap(),
                vertices[i + 1].clone().unwrap(),
            ]);
        }
        self.to_next_line();
        Ok(triangles)
    }

    pub fn parse_vertex(&self, face: &'a str, num_verticies: usize, num_texture_coords: usize, num_normals: usize) -> Result<Vertex, ImportError>
    {
        let parts = face.split('/').collect::<Vec<_>>();

        if (parts.len() < 1) || (parts.len() > 3)
        {
            return Err(self.create_error("Each face vertex requires 1 to 3 indexes"));
        }

        let vertex_index = self.parse_index(parts[0], num_verticies, "v")?;
        let mut normal_index = None;
        let mut texture_index = None;

        if (parts.len() >= 2) && !parts[1].is_empty()
        {
            texture_index = Some(self.parse_index(parts[1], num_texture_coords, "vt")?);
        }

        if (parts.len() >= 3) && !parts[2].is_empty()
        {
            normal_index = Some(self.parse_index(parts[2], num_normals, "vn")?);
        }

        Ok(Vertex{ vertex_index, normal_index, texture_index })
    }

    fn parse_index(&self, index: &'a str, num_items: usize, label: &str) -> Result<usize, ImportError>
    {
        let index = index.parse::<usize>().map_err(|_| self.create_error("Expected index"))?;

        if (index < 1) || (index > num_items)
        {
            return Err(self.create_error(&format!("Expected {} index in the range [1..{}] but got index {}", label, num_items, index)));
        }

        Ok(index - 1)
    }

    pub fn create_error(&self, err: &str) -> ImportError
    {
        ImportError(format!("{}:{}: {}: {}",
            self.context,
            self.cur_line_num,
            self.cur_line_parts[0],
            err))
    }

    fn to_next_line(&mut self)
    {
        loop
        {
            self.cur_line_num += 1;

            if self.cur_line_num >= self.lines.len()
            {
                return;
            }

            let mut line = self.lines[self.cur_line_num - 1];

            if let Some(pos) = line.find('#')
            {
                line = &line[0..pos];
            }

            if line.is_empty()
            {
                continue;
            }

            self.cur_line_parts = line.split(' ').collect();
            return;
        }
    }
}
