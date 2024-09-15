use std::collections::HashMap;
/// Hashmap like data structure made for storing Ip addresses and id's of the players to prevent 
/// Ip leaks. Clients should never be able to see each other's IP addresses.
pub struct ClientStore<Key,Val>{
    data:Vec<Val>,
    data_map: HashMap<Key,usize>
}


impl<Key,Val> ClientStore<Key, Val>
where Key: std::hash::Hash +Eq + Clone
{
    pub fn new() ->Self{
        Self{
            data:Vec::new(),
            data_map: HashMap::new(),
        }
    }



    pub fn push(&mut self, id:Key, channel:Val){
        self.data.push(channel);
        self.data_map.insert(id,self.data.len()-1);
    }


   
    pub fn get(&self, id: &Key) -> Option<&Val> {
        self.data_map.get(id).map(|&index| &self.data[index])
    }

    pub fn iter(&self)->std::slice::Iter<'_, Val> {
       self.data.iter() 
    }

    pub fn iter_mut(&mut self)->std::slice::IterMut<'_, Val>{
        self.data.iter_mut()
    }
    
    pub fn remove(&mut self, id: &Key) -> Option<Val> {
        if let Some(&index) = self.data_map.get(id) {
            let removed_value = self.data.swap_remove(index);

            if index < self.data.len() {
                let swapped_key = self.data_map.iter()
                    .find(|&(_, &v)| v == self.data.len())
                    .map(|(k, _)| k.clone())
                    .unwrap();
                self.data_map.insert(swapped_key, index);
            }

            self.data_map.remove(id);

            Some(removed_value)
        } else {
            None
        }
    }

}



#[cfg(test)]
mod tests{
    use super::*;

     #[test]
    fn test_remove() {
        let mut store = ClientStore::new();
        store.push(1, "Alice");
        store.push(2, "Bob");
        store.push(3, "Charlie");

        assert_eq!(store.remove(&2), Some("Bob"));
        let items: Vec<_> = store.iter().cloned().collect();
        assert_eq!(items, vec!["Alice", "Charlie"]);

        store.push(4, "David");
        assert_eq!(store.data_map.get(&4), Some(&2));
        assert_eq!(store.data[2], "David");
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut store = ClientStore::new();
        store.push(1, "Alice");
        store.push(2, "Bob");

        assert_eq!(store.remove(&3), None);
        let items: Vec<_> = store.iter().cloned().collect();
        assert_eq!(items, vec!["Alice", "Bob"]);
    }
}
