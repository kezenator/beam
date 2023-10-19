use std::collections::HashSet;
use std::hash::Hash;
use std::fmt::Debug;
use std::marker::PhantomData;

use imgui::TreeNodeFlags;

use crate::ui::{UiDisplay, UiEdit, UiRenderer};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ImageIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextureIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MaterialIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct GeomIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ObjectIndex(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnyIndex
{
    Image(ImageIndex),
    Texture(TextureIndex),
    Material(MaterialIndex),
    Geom(GeomIndex),
    Object(ObjectIndex),
}

pub trait Index
{
    fn from_usize(index: usize) -> Self;
    fn to_usize(&self) -> usize;
}

pub trait IndexedValue
{
    fn collect_indexes(&self, indexes: &mut HashSet<AnyIndex>);
    fn summary(&self) -> String;
}

impl Index for ImageIndex
{
    fn from_usize(index: usize) -> Self
    {
        ImageIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

impl Index for TextureIndex
{
    fn from_usize(index: usize) -> Self
    {
        TextureIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

impl Index for MaterialIndex
{
    fn from_usize(index: usize) -> Self
    {
        MaterialIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

impl Index for GeomIndex
{
    fn from_usize(index: usize) -> Self
    {
        GeomIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

impl Index for ObjectIndex
{
    fn from_usize(index: usize) -> Self
    {
        ObjectIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct IndexedVec<I, T>
where
    I: Index + Copy + Clone + Debug + Default + Eq + Hash,
    T: IndexedValue + Debug + Default
{
    items: Vec<T>,
    is_default: bool,
    phantom: PhantomData<I>,
}

impl<I, T> IndexedVec<I, T>
where
    I: Index + Copy + Clone + Debug + Default + Eq + Hash,
    T: IndexedValue + Debug + Default
{
    pub fn new() -> Self
    {
        let mut items = Vec::new();
        items.push(T::default());
        IndexedVec{ items, is_default: true, phantom: PhantomData::default() }
    }

    pub fn push(&mut self, item: T) -> I
    {
        if self.is_default
        {
            self.is_default = false;
            self.items[0] = item;
            I::from_usize(0)
        }
        else
        {
            self.items.push(item);            
            I::from_usize(self.items.len() - 1)
        }
    }

    pub fn push_default(&mut self) -> I
    {
        self.push(T::default())
    }

    pub fn get<'a> (&'a self, index: I) -> &'a T
    {
        self.items.get(index.to_usize()).unwrap()
    }

    pub fn get_mut<'a> (&'a mut self, index: I) -> &'a mut T
    {
        self.items.get_mut(index.to_usize()).unwrap()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    {
        self.items.iter()
    }
}

impl<I, T> Default for IndexedVec<I, T>
where
        I: Index + Copy + Clone + Debug + Default + Eq + Hash,
        T: IndexedValue + Debug + Default
{
    fn default() -> Self {
        Self::new()
    }
}

impl<I, T> UiDisplay for IndexedVec<I, T>
where
        I: Index + Copy + Clone + Debug + Default + Eq + Hash + UiDisplay,
        T: IndexedValue + Debug + Default + UiDisplay
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        let _child = ui.imgui.child_window(label);

        for (i, v) in self.items.iter().enumerate()
        {
            if ui.imgui.collapsing_header(i.to_string(), TreeNodeFlags::empty())
            {
                v.ui_display(ui, &i.to_string());
            }
        }
    }
}


impl<I, T> UiEdit for IndexedVec<I, T>
where
        I: Index + Copy + Clone + Debug + Default + Eq + Hash + UiEdit,
        T: IndexedValue + Debug + Default + UiEdit
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;
        let mut len = self.items.len();

        if let Some(_header) = ui.imgui.tree_node_config(&format!("{} ({} item{})###{}", label, len, if len == 1 { "" } else { "s" }, label))
            .frame_padding(true)
            .framed(true)
            .default_open(true)
            .push()
        {
            if ui.imgui.input_scalar("Count", &mut len).build()
            {
                result = true;

                while len < self.items.len()
                    && self.items.len() >= 2
                {
                    self.items.pop();
                }
                while len > self.items.len()
                {
                    self.items.push(T::default());
                }
            }

            for (i, v) in self.items.iter_mut().enumerate()
            {
                if let Some(_node) = ui.imgui.tree_node_config(format!("{} ({})###{}", i, v.summary(), i))
                    .default_open(true)
                    .framed(true)
                    .frame_padding(true)
                    .push()
                {
                    result |= v.ui_edit(ui, &i.to_string());
                }
            }
        }

        result
    }
}

impl UiDisplay for TextureIndex
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.imgui.label_text(label, self.0.to_string())
    }
}

impl UiEdit for TextureIndex
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        ui.imgui.input_scalar(label, &mut self.0).build()
    }
}

impl UiDisplay for GeomIndex
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.imgui.label_text(label, self.0.to_string())
    }
}

impl UiEdit for GeomIndex
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        ui.imgui.input_scalar(label, &mut self.0).build()
    }
}

impl UiDisplay for MaterialIndex
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.imgui.label_text(label, self.0.to_string())
    }
}

impl UiEdit for MaterialIndex
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        ui.imgui.input_scalar(label, &mut self.0).build()
    }
}

impl UiDisplay for ObjectIndex
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.imgui.label_text(label, self.0.to_string())
    }
}

impl UiEdit for ObjectIndex
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        ui.imgui.input_scalar(label, &mut self.0).build()
    }
}

