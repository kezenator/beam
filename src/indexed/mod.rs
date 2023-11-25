use std::any::{TypeId, Any};
use std::collections::{HashSet, HashMap};
use std::hash::Hash;
use std::fmt::Debug;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use imgui::TreeNodeFlags;
use itertools::Itertools;

use crate::ui::{UiDisplay, UiEdit, UiRenderer};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransformIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeomIndex(usize);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectIndex(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnyIndex
{
    Image(ImageIndex),
    Texture(TextureIndex),
    Transform(TransformIndex),
    Material(MaterialIndex),
    Geom(GeomIndex),
    Object(ObjectIndex),
}

pub trait Index: Debug + Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + Send + 'static
{
    type Value : IndexedValue;

    fn from_usize(index: usize) -> Self;
    fn to_usize(&self) -> usize;
}

pub trait IndexedValue: Debug + Default + Clone + UiDisplay + UiEdit + Send + 'static
{
    type Index: Index;

    fn collect_indexes(&self, indexes: &mut HashSet<AnyIndex>);
    fn summary(&self) -> String;
}

impl Index for TextureIndex
{
    type Value = crate::desc::edit::Texture;

    fn from_usize(index: usize) -> Self
    {
        TextureIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

impl Index for TransformIndex
{
    type Value = crate::desc::edit::Transform;

    fn from_usize(index: usize) -> Self
    {
        TransformIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

impl Index for MaterialIndex
{
    type Value = crate::desc::edit::Material;
    
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
    type Value = crate::desc::edit::Geom;
    
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
    type Value = crate::desc::edit::Object;
    
    fn from_usize(index: usize) -> Self
    {
        ObjectIndex(index)
    }

    fn to_usize(&self) -> usize
    {
        self.0
    }
}

pub trait IndexedCollectionVTable
{
    fn clone_vtable(&self) -> Box<dyn IndexedCollectionVTable>;
    fn clone_vec(&self, vec: &Box<dyn Any + Send>) -> Box<dyn Any + Send>;
    fn ui_display(&self, ui: &UiRenderer, label: &str, vec: &Box<dyn Any + Send>);
    fn ui_edit(&self, ui: &UiRenderer, label: &str, vec: &mut Box<dyn Any + Send>) -> bool;
}

pub struct IndexedCollectionVTableImpl<V: IndexedValue>
{
    phantom: PhantomData<V>,
}

impl<V: IndexedValue> IndexedCollectionVTableImpl<V>
{
    fn new() -> Self
    {
        IndexedCollectionVTableImpl { phantom: PhantomData }
    }

    fn downcast_ref<'a>(&self, vec: &'a Box<dyn Any + Send>) -> &'a IndexedVec<V>
    {
        &vec.downcast_ref::<IndexedVec<V>>().unwrap()
    }

    fn downcast_mut<'a>(&self, vec: &'a mut Box<dyn Any + Send>) -> &'a mut IndexedVec<V>
    {
        vec.downcast_mut::<IndexedVec<V>>().unwrap()
    }
}

impl<V: IndexedValue> IndexedCollectionVTable for IndexedCollectionVTableImpl<V>
{
    fn clone_vtable(&self) -> Box<dyn IndexedCollectionVTable>
    {
        Box::new(IndexedCollectionVTableImpl::<V>::new())
    }

    fn clone_vec(&self, vec: &Box<dyn Any + Send>) -> Box<dyn Any + Send>
    {
        Box::new(self.downcast_ref(vec).clone())
    }

    fn ui_display(&self, ui: &UiRenderer, label: &str, vec: &Box<dyn Any + Send>)
    {
        self.downcast_ref(vec).ui_display(ui, label);
    }

    fn ui_edit(&self, ui: &UiRenderer, label: &str, vec: &mut Box<dyn Any + Send>) -> bool
    {
        self.downcast_mut(vec).ui_edit(ui, label)
    }
}

pub struct IndexedCollectionEntry
{
    name: String,
    index: usize,
    key_index: TypeId,
    key_value: TypeId,
    vec: Box<dyn Any + Send>,
    vtable: Box<dyn IndexedCollectionVTable>,
}

pub struct IndexedCollection
{
    in_order: Vec<Rc<RefCell<IndexedCollectionEntry>>>,
    by_index: HashMap<TypeId, Rc<RefCell<IndexedCollectionEntry>>>,
    by_value: HashMap<TypeId, Rc<RefCell<IndexedCollectionEntry>>>,
}

unsafe impl Send for IndexedCollection
{
}

impl IndexedCollection
{
    pub fn new() -> Self
    {
        let in_order = Vec::new();
        let by_index = HashMap::new();
        let by_value = HashMap::new();

        IndexedCollection { in_order, by_index, by_value }
    }

    pub fn add_index<I: Index>(&mut self, index_name: &str)
    {
        let key_index = TypeId::of::<I>();
        let key_value = TypeId::of::<I::Value>();

        assert!(!self.by_index.contains_key(&key_index));
        assert!(!self.by_value.contains_key(&key_value));

        let vec = Box::new(IndexedVec::<I::Value>::new()) as Box<dyn Any + Send>;
        let vtable = Box::new(IndexedCollectionVTableImpl::<I::Value>::new());

        let entry = IndexedCollectionEntry
        {
            name: index_name.to_string(),
            index: self.in_order.len(),
            key_index,
            key_value,
            vec,
            vtable,
        };
        let rc = Rc::new(RefCell::new(entry));

        self.in_order.push(rc.clone());
        self.by_index.insert(key_index, rc.clone());
        self.by_value.insert(key_value, rc);
    }

    pub fn push<V: IndexedValue>(&mut self, value: V) -> V::Index
    {
        self.push_opt_name(value, None)
    }

    pub fn push_named<V: IndexedValue>(&mut self, value: V, name: String) -> V::Index
    {
        self.push_opt_name(value, Some(name))
    }

    pub fn push_opt_name<V: IndexedValue>(&mut self, value: V, name: Option<String>) -> V::Index
    {
        let key_value = TypeId::of::<V>();
        let entry = self.by_value.get_mut(&key_value).unwrap();
        entry.borrow_mut().vec.downcast_mut::<IndexedVec<V>>().unwrap().push_opt_named(value, name)
    }

    pub fn update_value<V: IndexedValue>(&mut self, index: V::Index, value: V)
    {
        let key_value = TypeId::of::<V>();
        let entry = self.by_value.get_mut(&key_value).unwrap();
        entry.borrow_mut().vec.downcast_mut::<IndexedVec<V>>().unwrap().update(index, value);
    }

    pub fn map_item<I: Index, F, V>(&self, index: I, func: F) -> V
        where F: FnOnce(&I::Value, &IndexedCollection) -> V
    {
        let key_index = TypeId::of::<I>();
        let entry = self.by_index.get(&key_index).unwrap();
        func(&entry.borrow().vec.downcast_ref::<IndexedVec<I::Value>>().unwrap().items[index.to_usize()].value.borrow(), self)
    }

    pub fn map_all<V: IndexedValue, F, R>(&self, func: F) -> Vec<R>
        where F: Fn(&V, &IndexedCollection) -> R
    {
        let key_value = TypeId::of::<V>();
        let entry = self.by_value.get(&key_value).unwrap();
        entry.borrow().vec.downcast_ref::<IndexedVec<V>>().unwrap().items.iter().map(move |e| func(&e.value.borrow(), self)).collect()
    }
}

impl Clone for IndexedCollection
{
    fn clone(&self) -> Self
    {
        let in_order = self.in_order.iter()
            .map(|e|
            {
                let e = e.borrow();

                let entry = IndexedCollectionEntry
                {
                    name: e.name.clone(),
                    index: e.index,
                    key_index: e.key_index,
                    key_value: e.key_value,
                    vec: e.vtable.clone_vec(&e.vec),
                    vtable: e.vtable.clone_vtable(),
                };
                Rc::new(RefCell::new(entry))
            })
            .collect_vec();

        let by_index = in_order.iter()
            .map(|e| (e.borrow().key_index, e.clone()))
            .collect();

        let by_value = in_order.iter()
            .map(|e| (e.borrow().key_value, e.clone()))
            .collect();

        Self { in_order, by_index, by_value }
    }
}

impl UiDisplay for IndexedCollection
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        let _id = ui.imgui.push_id(label);
        if ui.imgui.collapsing_header(label, TreeNodeFlags::empty())
        {
            ui.imgui.indent();
            for i in self.in_order.iter()
            {
                let i = i.borrow();
                let _i_id = ui.imgui.push_id_usize(i.index);
                if ui.imgui.collapsing_header(&i.name, TreeNodeFlags::empty())
                {
                    ui.imgui.indent();

                    i.vtable.ui_display(ui, &i.name, &i.vec);

                    ui.imgui.unindent();
                }
            }
            ui.imgui.unindent();
        }
    }
}

impl UiEdit for IndexedCollection
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut result = false;

        let _id = ui.imgui.push_id(label);
        if ui.imgui.collapsing_header(label, TreeNodeFlags::empty())
        {
            ui.imgui.indent();
            for i in self.in_order.iter()
            {
                let i = &mut *i.borrow_mut();
                let _i_id = ui.imgui.push_id_usize(i.index);
                if ui.imgui.collapsing_header(&i.name, TreeNodeFlags::empty())
                {
                    ui.imgui.indent();

                    result |= i.vtable.ui_edit(ui, &i.name, &mut i.vec);

                    ui.imgui.unindent();
                }
            }
            ui.imgui.unindent();
        }
        result
    }
}

#[derive(Clone, Debug)]
struct IndexedVecEntry<V: IndexedValue>
{
    value: RefCell<V>,
    name: Option<String>,
    is_default: bool,
}

#[derive(Clone, Debug)]
pub struct IndexedVec<V: IndexedValue>
{
    items: Vec<IndexedVecEntry<V>>,
}

impl<V: IndexedValue> IndexedVec<V>
{
    pub fn new() -> Self
    {
        let mut items = Vec::new();
        items.push(IndexedVecEntry { value: RefCell::new(V::default()), name: None, is_default: true });
        IndexedVec{ items }
    }

    pub fn push(&mut self, item: V) -> V::Index
    {
        self.push_internal(item, None)
    }

    pub fn push_opt_named(&mut self, item: V, name: Option<String>) -> V::Index
    {
        self.push_internal(item, name)
    }

    pub fn push_default(&mut self) -> V::Index
    {
        self.push_internal(V::default(), None)
    }

    pub fn update(&mut self, i: V::Index, v: V)
    {
        let entry = &mut self.items[i.to_usize()];
        entry.is_default = false;
        *entry.value.borrow_mut() = v;
    }

    fn push_internal(&mut self, item: V, opt_name: Option<String>) -> V::Index
    {
        if self.items.len() == 1
            && self.items[0].is_default
        {
            self.items[0].is_default = false;
            self.items[0].name = opt_name;
            self.items[0].value.replace(item);
            V::Index::from_usize(0)
        }
        else
        {
            self.items.push(IndexedVecEntry { value: RefCell::new(item), name: opt_name, is_default: false });
            V::Index::from_usize(self.items.len() - 1)
        }
    }
}

impl<V: IndexedValue> Default for IndexedVec<V>
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl<V: IndexedValue> UiDisplay for IndexedVec<V>
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        let _id = ui.imgui.push_id(label);

        for (i, e) in self.items.iter().enumerate()
        {
            let index_str = i.to_string();

            if ui.imgui.collapsing_header(&index_str, TreeNodeFlags::empty())
            {
                e.value.borrow().ui_display(ui, &index_str);
            }
        }
    }
}


impl<V: IndexedValue> UiEdit for IndexedVec<V>
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
                    self.push_default();
                }
            }

            for (i, e) in self.items.iter_mut().enumerate()
            {
                let mut v = e.value.borrow_mut();

                let display_summary = e.name.clone().unwrap_or_else(|| v.summary());

                if let Some(_node) = ui.imgui.tree_node_config(format!("{} ({})###{}", i, display_summary, i))
                    .default_open(true)
                    .framed(true)
                    .frame_padding(true)
                    .push()
                {
                    let changed = v.ui_edit(ui, &i.to_string());
                    result |= changed;

                    if changed
                    {
                        e.is_default = false;
                    }
                }
            }
        }

        result
    }
}

impl<T> UiDisplay for T
    where T: Index
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.imgui.label_text(label, self.to_usize().to_string())
    }
}

impl<T> UiEdit for T
    where T: Index
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut as_usize = self.to_usize();

        if ui.imgui.input_scalar(label, &mut as_usize).build()
        {
            *self = T::from_usize(as_usize);
            return true;
        }
        false
    }
}

impl<T> UiDisplay for Option<T>
    where T: Index
{
    fn ui_display(&self, ui: &UiRenderer, label: &str)
    {
        ui.imgui.label_text(label, self.map(|s| s.to_usize().to_string()).unwrap_or_else(|| "<None>".to_string()))
    }
}

impl<T> UiEdit for Option<T>
    where T: Index
{
    fn ui_edit(&mut self, ui: &UiRenderer, label: &str) -> bool
    {
        let mut as_usize = self.map(|s| s.to_usize()).unwrap_or(usize::MAX);

        if ui.imgui.input_scalar(label, &mut as_usize).build()
        {
            *self = Some(T::from_usize(as_usize));
            return true;
        }
        false
    }
}
