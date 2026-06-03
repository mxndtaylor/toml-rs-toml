use std::hash::Hash;
use indexmap::IndexMap;
use indexmap::map::{MutableKeys, Entry};
use crate::internal::api::{EntryCore, OccupiedEntryCore};

struct BucketProxy<'a, K, I>
where
    K: Hash + Eq,
{
    // I was thinking of have a reference to the OccupiedEntry here,
    // and a reference to the bucket somehow?? or just the original indexmap...
    // then we wouldn't need a reference to the OccupiedEntry...
    // and it would break the idea that we're just scooping out
    // all the impl from inline_table.rs and table.rs...
    // so we might need to re-think a little.
    //
    // well... technically we can still do it.
    // the Entry structs and impls for Table and InlineTable do not
    // expose their internal indexmap-dependent representation to public api
    // so we could just not have it reference indexmap's structs anymore.
    // that could solve this, as we could then just have our own types supporting it
    key: &'a K,
    items: IndexMap<K, I>,
}

impl<'a, K, I> OccupiedEntryCore<K, I> for BucketProxy<'a, K, I> {
    fn get_full(&self) -> (&K, &I) {
        self.items.get_key_value(self.key).unwrap()
    }

    fn get_full_mut(&mut self) -> (&K, &mut I) {
        self.items.get_full_mut(self.key).map(|(_, k, i)| (k, i)).unwrap()
    }

    fn get_full_mut2(&mut self) -> (&mut K, &mut I) {
        self.items.get_full_mut2(self.key).map(|(_, k, i)| (k, i)).unwrap()
    }

    fn occ_insert(&mut self, value: I) -> I {
        match self.items.entry(self.key) {
            Entry::Occupied(mut e) => e.insert(value),
            _ => panic!("expected occupied entry, but found vacant entry"),
        }
    }

    fn remove(&mut self) -> (K, I) {
        self.items.shift_remove_entry(self.key).unwrap()
    }
}