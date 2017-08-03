    fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.ch, &self.map.read().unwrap())
    }
