use crate::Iter;

pub(crate) trait EntryCore<K, I> : Sized {
    /// gets the key as represented by this entry
    fn key(&self) -> &K;
    /// if there was a previous value, it is returned as Ok()
    /// otherwise, returns an Err() with mutable borrow of value as inserted
    fn insert(&mut self, value: I) -> Result<I, &mut I>;
    /// if there was a value, it is returned as Some()
    /// otherwise returns None
    fn remove(&mut self) -> Option<(K, I)>;
}

pub(crate) trait OccupiedEntryCore<K, I> : EntryCore<K, I> {
    fn get_full(&self) -> (&K, &I);
    fn get_full_mut(&mut self) -> (&K, &mut I);
    fn get_full_mut2(&mut self) -> (&mut K, &mut I);
    fn key(&self) -> &K {
        self.get_full().0
    }
    fn get(&self) -> &I {
        self.get_full().1
    }
    fn get_mut(&mut self) -> &mut I {
        self.get_full_mut().1
    }
}

pub(crate) trait VacantEntryCore<K, I> : EntryCore<K, I> {
    fn remove(&mut self) -> Option<(K, I)> {
        None
    }
}

pub(crate) trait EntryBucket<O, V, K, I> : Sized
where
    O: OccupiedEntryCore<K, I>,
    V: VacantEntryCore<K, I>,
{
    fn get(self) -> Result<O, V>;
    fn get_ref(&self) -> Result<&O, &V>;
    fn get_mut(&mut self) -> Result<&mut O, &mut V>;
    fn remove(self) -> Result<O, V>;
    fn or_insert(&mut self, default: V) -> &mut V {
        self.or_insert_with(|| { default })
    }
    fn or_insert_with<F>(&mut self, default_factory: F) -> &mut V
    where
        F: FnOnce() -> V,
    {
        match self.get_mut() {
            Ok(occ) => occ.get_mut(),
            Err(vac) => vac.insert(default_factory()),
        }
    }
    fn is_occupied(&self) -> bool {
        self.get_ref().is_ok()
    }
    fn is_vacant(&self) -> bool {
        !self.is_occupied()
    }
}

impl<E, O, V, K, I> EntryCore<K, I> for E
where
    E: EntryBucket<O, V, K, I>,
    O: OccupiedEntryCore<K, I>,
    V: VacantEntryCore<K, I>,
{
    fn key(&self) -> &K {
        let entry = match self.get_ref() {
            Ok(occ) => occ,
            Err(vac) => vac,
        };
        entry.key()
    }

    fn insert(&mut self, value: I) -> &mut I {
        let entry = match self.get_mut() {
            Ok(occ) => occ,
            Err(vac) => vac,
        };
        entry.insert(value)
    }

    fn remove(&mut self) -> Option<(K, I)> {
        let entry = match self.get_mut() {
            Ok(occ) => occ,
            Err(vac) => vac,
        };
        entry.remove()
    }
}

pub(crate) trait TableCore<E, O, V, K, I> : Sized
where
    E: EntryBucket<O, V, K, I>,
    O: OccupiedEntryCore<K, I>,
    V: VacantEntryCore<K, I>,
{
    fn pair(&self, key: &K) -> &E;
    fn pair_mut(&self, key: &K) -> &mut E;
    fn pair_iter<It>(&self) -> It
    where
        It: Iterator<Item = &O>;
    fn pair_iter_mut<It>(&mut self) -> It
    where
        It: Iterator<Item = &mut O>;
    fn remove_pair(&mut self, key: &K) -> E;
}

pub(crate) trait Table<K, I> : Sized {
    fn get_key_value(&self, key: &K) -> Option<(&K, &I)>;
    fn get_key_value_mut(&mut self, key: &K) -> Option<(&mut K, &mut I)>;
    fn iter(&self) -> impl Iterator<Item = (&K, &I)>;
    fn iter_mut(&mut self) -> impl Iterator<Item = (&mut K, &mut I)>;
    fn insert(&mut self, key: K, value: I) -> Option<I>;
    fn remove_key_value(&mut self, key: &K) -> Option<(K, I)>;

    fn get(&self, key: &K) -> Option<&I> {
        self.get_key_value(key).and_then(|_, v| v)
    }
    fn get_mut(&mut self, key: &K) -> Option<&mut I> {
        self.get_key_value_mut(key).and_then(|_, v| v)
    }
    fn key(&self, key: &K) -> Option<&K> {
        self.get_key_value(key).and_then(|_, v| v)
    }
    fn key_mut(&mut self, key: &K) -> Option<&mut K> {
        self.get_key_value_mut(key).and_then(|_, v| v)
    }
    fn remove(&mut self, key: &K) -> Option<I> {
        self.remove_key_value(key).and_then(|(_, v)| v)
    }
    fn contains_key(&self, key: &K) -> bool {
        self.contains_with(key, |_| true)
    }
    fn contains_with<F>(&self, key: &K, predicate: F) -> bool
    where
        F: FnMut(&I) -> bool
    {
        match self.get(key) {
            Some(value) => predicate(value),
            None => false,
        }
    }
    fn len(&self) -> usize {
        self.iter().count()
    }
    fn retain<F>(&mut self, keep: F)
    where
        F: FnMut(&K, &mut I) -> bool
    {
        for (k, v) in self.iter_mut() {
            if !keep(k, v) {
                self.remove(k);
            }
        }
    }
    fn clear(&mut self) {
        self.retain(|_, &mut _| false);
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, I)>,
    {
        for (key, value) in iter {
            let key = key.into();
            let value = value.into();
            self.insert(key, value);
        }
    }
}


impl<T, E, O, V, K, I> Table<K, I> for T
where
    T: TableCore<E, O, V, K, I>,
    E: EntryBucket<O, V, K, I>,
    O: OccupiedEntryCore<K, I>,
    V: VacantEntryCore<K, I>,
{
    fn get_key_value(&self, key: &K) -> Option<(&K, &I)> {
        self.pair(key)
            .get_ref()
            .ok()
            .and_then(|value| Some(value.get_full()))
    }

    fn get_key_value_mut(&mut self, key: &K) -> Option<(&mut K, &mut I)> {
        self.pair_mut(key)
            .get_mut()
            .ok()
            .and_then(|value| Some(value.get_full_mut2()))
    }

    fn iter(&self) -> impl Iterator<Item=(&K, &I)> {
        self.pair_iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item=(&mut K, &mut I)> {
        self.pair_iter_mut()
    }

    fn insert(&mut self, key: K, value: I) -> Option<I> {
        self.pair_mut(key)
    }

    fn remove_key_value(&mut self, key: &K) -> Option<(K, I)> {
        match self.remove_pair(key) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}
