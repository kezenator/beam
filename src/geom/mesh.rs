use crate::math::Scalar;
use crate::vec::Point3;
use crate::geom::{Aabb, BoundingSurface, Surface, Triangle};
use crate::intersection::SurfaceIntersection;
use crate::ray::{Ray, RayRange};

#[derive(Clone)]
struct Split
{
    bounds: Aabb,
    sub_tree: Octree,
}

#[derive(Clone)]
enum Octree
{
    Leaf(Vec<usize>),
    Split(Vec<Split>),
}

#[derive(Clone)]
pub struct Mesh
{
    triangles: Vec<Triangle>,
    octree: Octree,
}

impl Mesh
{
    pub fn new(triangles: Vec<Triangle>) -> Self
    {
        let octree = Octree::new(&triangles);
        //let octree = Octree::Leaf((0..triangles.len()).collect());
        println!("Created Octree: {} triangles => {} nodes {} depth", triangles.len(), octree.count_nodes(), octree.max_depth());
        //assert!(false);

        Mesh { triangles, octree }
    }
}

impl Surface for Mesh
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        let mut range = range.clone();
        let mut closest = None;

        self.octree.closest_intersection_in_range(&self.triangles, ray, &mut range, &mut closest);

        closest
    }
}

impl Octree
{
    fn new(triangles: &Vec<Triangle>) -> Self
    {
        let tri_bounds = triangles.iter().map(|t| t.bounds()).collect::<Vec<_>>();

        let full_bounds = tri_bounds.iter().fold(tri_bounds[0].clone(), |a, b| a.union(b));

        //println!("Building Octree for {} triangles within {:?}", triangles.len(), full_bounds);

        build_octree(triangles, &tri_bounds, full_bounds, "root")
    }

    fn closest_intersection_in_range<'r>(&self, triangles: &Vec<Triangle>, ray: &'r Ray, range: &mut RayRange, closest: &mut Option<SurfaceIntersection<'r>>)
    {
        match self
        {
            Octree::Leaf(indexes) =>
            {
                for i in indexes.iter()
                {
                    if let Some(intersection) = triangles[*i].closest_intersection_in_range(ray, &range)
                    {
                        range.set_max(intersection.distance);
                        *closest = Some(intersection);
                    }
                }
            },
            Octree::Split(splits) =>
            {
                for split in splits.iter()
                {
                    if split.bounds.may_intersect_in_range(ray, range)
                    {
                        split.sub_tree.closest_intersection_in_range(triangles, ray, range, closest);
                    }
                }
            },
        }
    }

    fn count_nodes(&self) -> usize
    {
        match self
        {
            Octree::Leaf(_) => 1,
            Octree::Split(splits) => 1 + splits.iter().map(|s| s.sub_tree.count_nodes()).sum::<usize>(),
        }
    }

    fn max_depth(&self) -> usize
    {
        match self
        {
            Octree::Leaf(_) => 1,
            Octree::Split(splits) => 1 + splits.iter().map(|s| s.sub_tree.max_depth()).max().unwrap_or(1),
        }
    }
}

fn build_octree(triangles: &Vec<Triangle>, tri_bounds: &Vec<Aabb>, cur_bounds: Aabb, name: &str) -> Octree
{
    let indexes_in_cur_bounds = tri_bounds
        .iter()
        .enumerate()
        .filter(|(_, tri_bound)| cur_bounds.intersects(tri_bound))
        .map(|(i, _)| i)
        .collect::<Vec<usize>>();

    //println!("Level: {} Contains: {} Within: {:?}", name, indexes_in_cur_bounds.len(), cur_bounds);
    //assert!(name.len() < 80);

    if indexes_in_cur_bounds.len() > 10
    {
        // Try and split along longest axis
        let x_size = (cur_bounds.max.x - cur_bounds.min.x).abs();
        let y_size = (cur_bounds.max.y - cur_bounds.min.y).abs();
        let z_size = (cur_bounds.max.z - cur_bounds.min.z).abs();
        //println!("Level: {} Size: {}/{}/{}", name, x_size, y_size, z_size);

        if (x_size >= y_size) && (x_size >= z_size)
        {
            return create_split(
                triangles, tri_bounds, cur_bounds, &indexes_in_cur_bounds,
            &format!("{}.x", name),
                |v| v.x,
                |v, s| Point3::new(s, v.y, v.z));
        }
        else if (y_size >= x_size) && (y_size >= z_size)
        {
            return create_split(
                triangles, tri_bounds, cur_bounds, &indexes_in_cur_bounds,
            &format!("{}.y", name),
                |v| v.y,
                |v, s| Point3::new(v.x, s, v.z));
        }
        else
        {
            return create_split(
                triangles, tri_bounds, cur_bounds, &indexes_in_cur_bounds,
            &format!("{}.z", name),
                |v| v.z,
                |v, s| Point3::new(v.x, v.y, s));
        }
    }

    //println!("Level: {} => Leaf {}", name, indexes_in_cur_bounds.len());

    Octree::Leaf(indexes_in_cur_bounds)
}

fn create_split<Extract, Combine>(triangles: &Vec<Triangle>, tri_bounds: &Vec<Aabb>, cur_bounds: Aabb, indexes_in_cur_bounds: &Vec<usize>, name: &str, extract: Extract, combine: Combine) -> Octree
    where Extract: Fn(&Point3) -> Scalar,
        Combine: Fn(&Point3, Scalar) -> Point3
{
    let num_cur = indexes_in_cur_bounds.len();

    //println!("Level: {} => Create split for {} in {:?}", name, num_cur, cur_bounds);

    let extract_min = extract(&cur_bounds.min);
    let extract_max = extract(&cur_bounds.max);
    let mid = extract_min + 0.5 * (extract_max - extract_min);
    assert!(mid >= extract_min);
    assert!(mid <= extract_max);

    let bounds_0 = Aabb::new(cur_bounds.min, combine(&cur_bounds.max, mid));
    let bounds_1 = Aabb::new(combine(&cur_bounds.min, mid), cur_bounds.max);

    let indexes_in_0 = indexes_in_cur_bounds.iter()
        .filter(|i| bounds_0.intersects(&tri_bounds[**i]))
        .cloned()
        .collect::<Vec<usize>>();
    let indexes_in_1 = indexes_in_cur_bounds.iter()
        .filter(|i| bounds_1.intersects(&tri_bounds[**i]))
        .cloned()
        .collect::<Vec<usize>>();

    let num_0 = indexes_in_0.len();
    let num_1 = indexes_in_1.len();

    //println!("Level: {} => split into {}/{}", name, num_0, num_1);

    if (num_0 == 0) && (num_1 == num_cur)
    {
        // Split 0 is is empty - just return split 1
        return build_octree(triangles, tri_bounds, bounds_1, name);
    }
    else if (num_0 == num_cur) && (num_1 == 0)
    {
        // Split 1 is empty - just return split 0
        return build_octree(triangles, tri_bounds, bounds_0, name);
    }
    else if (num_0 == num_cur) || (num_1 == num_cur)
    {
        // TODO - bad split - just return a leaf
        return Octree::Leaf(indexes_in_cur_bounds.clone());
    }

    // Looks like we have a valid split - create the two octrees below this

    let octree_0 = build_octree(triangles, tri_bounds, bounds_0.clone(), &format!("{}.0", name));
    let octree_1 = build_octree(triangles, tri_bounds, bounds_1.clone(), &format!("{}.1", name));

    let split_0 = Split{ bounds: bounds_0, sub_tree: octree_0, };
    let split_1 = Split{ bounds: bounds_1, sub_tree: octree_1, };

    Octree::Split(vec![split_0, split_1])
}