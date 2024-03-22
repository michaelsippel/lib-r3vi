use crate::view::View;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub enum ListDiff<T>
where T: Clone + Send + Sync + 'static
{
    Clear,
    Remove(usize),
    Insert{ idx: usize, val: T },
    Update{ idx: usize, val: T },
}

pub trait ListView<Item>: View<Msg = ListDiff<Item>>
where Item: Clone + Send + Sync + 'static
{
    fn len(&self) -> Option<usize>;
    fn get(&self, idx: &usize) -> Option<Item>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait ListViewExt<T>: ListView<T>
where T: Clone + Send + Sync + 'static
{
    fn iter<'a>(&'a self) -> ListViewIter<'a, T, Self> {
        ListViewIter { _phantom: std::marker::PhantomData, view: self, cur: 0 }
    }
}

impl<T, V: ListView<T> + ?Sized> ListViewExt<T> for V
where T: Clone + Send + Sync + 'static
{}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ListViewIter<'a, T, V>
where
    T: Clone + Send + Sync + 'static,
    V: ListView<T> + ?Sized,
{
    _phantom: std::marker::PhantomData<T>,
    view: &'a V,
    cur: usize,
}

impl<'a, T, V> Iterator for ListViewIter<'a, T, V>
where
    T: Clone + Send + Sync + 'static,
    V: ListView<T> + ?Sized,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.cur;
        self.cur += 1;
        self.view.get(&i)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::sync::RwLock;
use std::{ops::Deref, sync::Arc};

impl<Item: Clone + Send + Sync + 'static, V: ListView<Item> + ?Sized> ListView<Item> for RwLock<V> {
    fn get(&self, idx: &usize) -> Option<Item> {
        self.read().unwrap().get(idx)
    }

    fn len(&self) -> Option<usize> {
        self.read().unwrap().len()
    }
}

impl<Item: Clone + Send + Sync + 'static, V: ListView<Item> + ?Sized> ListView<Item> for Arc<V> {
    fn get(&self, idx: &usize) -> Option<Item> {
        self.deref().get(idx)
    }

    fn len(&self) -> Option<usize> {
        self.deref().len()
    }
}

impl<Item: Clone + Send + Sync + 'static, V: ListView<Item>> ListView<Item> for Option<V> {
    fn get(&self, idx: &usize) -> Option<Item> {
        (self.as_ref()? as &V).get(idx)
    }

    fn len(&self) -> Option<usize> {
        if let Some(v) = self.as_ref() {
            v.len()
        } else {
            Some(0)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

