use std::any::Any;

struct World {
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
}

trait ComponentVec {
    fn push_none(&mut self);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> ComponentVec for Vec<Option<T>> {
    fn push_none(&mut self) {
        self.push(None)
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn std::any::Any
    }
}

impl World {
    fn new() -> Self {
        Self {
            entities_count: 0,
            component_vecs: Vec::new(),
        }
    }

    fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }
        self.entities_count += 1;
        entity_id
    }

    fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<Vec<Option<ComponentType>>>()
            {
                component_vec[entity] = Some(component);
                return;
            }
        }

        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);

        for _ in 0..self.entities_count {
            new_component_vec.push(None);
        }

        new_component_vec[entity] = Some(component);
        self.component_vecs.push(Box::new(new_component_vec));
    }

    fn borrow_component_vec<ComponentType: 'static>(&self) -> Option<&Vec<Option<ComponentType>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<Vec<Option<ComponentType>>>()
            {
                return Some(component_vec);
            }
        }

        None
    }
}

mod test {
    use super::*;

    struct Heath(i32);
    struct Name(String);
    struct No(Option<String>);

    #[test]
    fn esc_test() {
        let mut world = World::new();
        let entity = world.new_entity();
        world.add_component_to_entity(entity, Heath(100));
        world.add_component_to_entity(entity, Name(String::from("Somebody")));
        world.add_component_to_entity(entity, No(None));

        if let Some(healths) = world.borrow_component_vec::<Heath>() {
            for h in healths.iter().filter_map(|f| f.as_ref()) {
                println!("{:?}", h.0)
            }
        }

        let zip = world
            .borrow_component_vec::<Heath>()
            .unwrap()
            .iter()
            .zip(world.borrow_component_vec::<Name>().unwrap().iter());

        for (h, n) in zip.filter_map(|(h, n)| Some((h.as_ref()?, n.as_ref()?))) {
            println!("{:?}-{:?}", h.0, n.0)
        }
    }
}
