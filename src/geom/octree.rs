use crate::geom::{Aabb, AabbBoundedSurface, BoundingSurface, Surface};
use crate::intersection::SurfaceIntersection;
use crate::math::Scalar;
use crate::ray::{Ray, RayRange};
use crate::vec::Point3;

#[derive(Clone)]
pub struct Octree<S: AabbBoundedSurface + Clone + 'static>
{
    bounds: Aabb,
    items: Vec<S>,
    tree: OctreeNode,
}

impl<S: AabbBoundedSurface + Clone + 'static> Octree<S>
{
    pub fn new(items: Vec<S>, target_leaf_size: usize) -> Self
    {
        assert!(!items.is_empty());

        let mut bounds = items[0].get_bounding_aabb();

        let mut item_bounds = Vec::new();
        item_bounds.reserve(items.len());

        for item in items.iter()
        {
            let item_bound = item.get_bounding_aabb();
            bounds = bounds.union(&item_bound);
            item_bounds.push(item_bound);
        }

        let tree = build_octree(&items, &item_bounds, bounds.clone(), "root");

        let result = Octree { bounds, items, tree };

        let stats = result.get_stats();
        println!("Created octree: {} items, {} nodes, {} total-item-refs, {} depth, ({}..{}) leaf-size, {} target-leaf-size",
            result.items.len(), stats.num_nodes, stats.total_item_refs, stats.max_depth, stats.smallest_node, stats.largest_node, target_leaf_size);

        result
    }

    pub fn get_stats(&self) -> OctreeStats
    {
        let mut result = OctreeStats::new();
        self.tree.add_stats(&mut result, 1);
        result
    }
}

impl<S: AabbBoundedSurface + Clone + 'static> Surface for Octree<S>
{
    fn closest_intersection_in_range<'r>(&self, ray: &'r Ray, range: &RayRange) -> Option<SurfaceIntersection<'r>>
    {
        if self.bounds.may_intersect_in_range(ray, range)
        {
            let mut range = range.clone();
            let mut closest = None;

            self.tree.closest_intersection_in_range(&self.items, ray, &mut range, &mut closest);

            closest
        }
        else
        {
            None
        }
    }
}

impl<S: AabbBoundedSurface + Clone + 'static> AabbBoundedSurface for Octree<S>
{
    fn get_bounding_aabb(&self) -> Aabb
    {
        self.bounds.clone()
    }
}

#[derive(Clone)]
enum OctreeNode
{
    Leaf(Vec<usize>),
    Split(Vec<Split>),
}

impl OctreeNode
{
    fn add_stats(&self, stats: &mut OctreeStats, depth: usize)
    {
        match self
        {
            OctreeNode::Leaf(indexes) =>
            {
                stats.add_node(depth, Some(indexes.len()));
            },
            OctreeNode::Split(splits) =>
            {
                stats.add_node(depth, None);

                for split in splits.iter()
                {
                    split.sub_tree.add_stats(stats, depth + 1);
                }
            },
        }
    }

    fn closest_intersection_in_range<'r, S:AabbBoundedSurface + Clone + 'static>(&self, items: &Vec<S>, ray: &'r Ray, range: &mut RayRange, closest: &mut Option<SurfaceIntersection<'r>>)
    {
        match self
        {
            OctreeNode::Leaf(indexes) =>
            {
                for i in indexes.iter()
                {
                    if let Some(intersection) = items[*i].closest_intersection_in_range(ray, &range)
                    {
                        range.set_max(intersection.distance);
                        *closest = Some(intersection);
                    }
                }
            },
            OctreeNode::Split(splits) =>
            {
                for split in splits.iter()
                {
                    if split.bounds.may_intersect_in_range(ray, range)
                    {
                        split.sub_tree.closest_intersection_in_range(items, ray, range, closest);
                    }
                }
            },
        }
    }
}

#[derive(Clone)]
struct Split
{
    bounds: Aabb,
    sub_tree: OctreeNode,
}

pub struct OctreeStats
{
    num_nodes: usize,
    max_depth: usize,
    smallest_node: usize,
    largest_node: usize,
    total_item_refs: usize,
}

impl OctreeStats
{
    fn new() -> Self
    {
        OctreeStats
        {
            num_nodes: 0,
            max_depth: 0,
            smallest_node: usize::max_value(),
            largest_node: 0,
            total_item_refs: 0,
        }
    }

    fn add_node(&mut self, depth: usize, size: Option<usize>)
    {
        self.num_nodes += 1;
        self.max_depth = self.max_depth.max(depth);
        if let Some(size) = size
        {
            self.smallest_node = self.smallest_node.min(size);
            self.largest_node = self.largest_node.max(size);
            self.total_item_refs += size;
        }
    }
}

fn build_octree<S: AabbBoundedSurface + Clone + 'static>(triangles: &Vec<S>, tri_bounds: &Vec<Aabb>, cur_bounds: Aabb, name: &str) -> OctreeNode
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

    OctreeNode::Leaf(indexes_in_cur_bounds)
}

fn create_split<S, Extract, Combine>(triangles: &Vec<S>, tri_bounds: &Vec<Aabb>, cur_bounds: Aabb, indexes_in_cur_bounds: &Vec<usize>, name: &str, extract: Extract, combine: Combine) -> OctreeNode
    where S: AabbBoundedSurface + Clone + 'static,
        Extract: Fn(&Point3) -> Scalar,
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
        return OctreeNode::Leaf(indexes_in_cur_bounds.clone());
    }

    // Looks like we have a valid split - create the two octrees below this

    let octree_0 = build_octree(triangles, tri_bounds, bounds_0.clone(), &format!("{}.0", name));
    let octree_1 = build_octree(triangles, tri_bounds, bounds_1.clone(), &format!("{}.1", name));

    let split_0 = Split{ bounds: bounds_0, sub_tree: octree_0, };
    let split_1 = Split{ bounds: bounds_1, sub_tree: octree_1, };

    OctreeNode::Split(vec![split_0, split_1])
}