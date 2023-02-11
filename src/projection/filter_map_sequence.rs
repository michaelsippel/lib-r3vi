use {
    crate::{
        view::{
            OuterViewPort,
            sequence::SequenceView,
        },
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: 'static> OuterViewPort<dyn SequenceView<Item = Item>> {
    pub fn filter_map<
        DstItem: Clone + 'static,
        F: Fn(&Item) -> Option<DstItem> + Send + Sync + 'static,
    >(
        &self,
        f: F,
    ) -> OuterViewPort<dyn SequenceView<Item = DstItem>> {
        self.map(f)
            .filter(|x| x.is_some())
            .map(|x| x.clone().unwrap())
    }
}

