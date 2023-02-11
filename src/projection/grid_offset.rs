use {
    crate::{
        view::{
            OuterViewPort,
            grid::GridView,
        }
    },
    cgmath::Vector2,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item> OuterViewPort<dyn GridView<Item = Item>>
where
    Item: 'static,
{
    pub fn offset(&self, offset: Vector2<i16>) -> OuterViewPort<dyn GridView<Item = Item>> {
        self.map_key(move |pt| pt + offset, move |pt| Some(pt - offset))
    }
}

