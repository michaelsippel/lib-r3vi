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

