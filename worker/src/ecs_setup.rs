use quicksilver::Quicksilver;
use std::collections::HashMap;

type TypeName = String;

#[derive(Default, Debug, Quicksilver)]
pub struct OriginTarget(pub u32, pub u32);
#[derive(Default, Debug, Quicksilver)]
pub struct EntityComponent(pub u32, pub String);

#[derive(Default, Debug, Quicksilver)]
pub struct SerializedState {
    // TypeName, Vec<(EntityId, ComponentPayload)>
    pub components: HashMap<TypeName, Vec<EntityComponent>>,
    // TypeName, Vec<(Origin, Target)>
    pub relations: HashMap<TypeName, Vec<OriginTarget>>,
}

macro_rules! generate_register {
    (@rel $world:ident $ty:tt $flags:tt) => {
        $world.register_relation_flags::<$ty>($flags);
    };
    (@rel $world:ident $ty:tt) => {
        $world.register_relation::<$ty>();
    };
    (Components($($components:tt $([persist])?),*),
     Relations($($relations:tt $(($flags:expr))? $([persist])? ),*)) => {
        pub fn register_components(world: &mut World) {
            $(world.register_component::<$components>();)*
            $(generate_register!(@rel world $relations $($flags)?);)*
        }
    };
}

macro_rules! generate_re_register {
    (@rel $world:ident $ty:tt $flags:tt) => {
        $world.re_register_relation::<$ty>()?;
    };
    (@rel $world:ident $ty:tt) => {
        $world.re_register_relation::<$ty>()?;
    };
    (Components($($components:tt $([persist])?),*),
     Relations($($relations:tt $(($flags:expr))? $([persist])? ),*)) => {
        pub fn re_register_components(world: &mut World) ->
            Result<(), ::froql::world::ReregisterError> {
            unsafe {
                $(world.re_register_component::<$components>()?;)*
                $(generate_register!(@rel world $relations $($flags)?);)*
            }
            Ok(())
        }
    };
}

macro_rules! generate_save {
    (@rel $world:ident $state:ident $ty:tt persist) => {
        $state.relations.insert(
            type_name::<$ty>().to_string(),
            $world
                .bookkeeping
                .relation_pairs(TypeId::of::<Relation<$ty>>())
                .into_iter()
                .map(|(o, t)| OriginTarget(o.id.0, t.id.0))
                .collect(),
        );
    };
    (@rel $world:ident $state:ident $ty:tt ) => {};
    (@comp $world:ident $state:ident $ty:tt persist) => {
        let mut buffer = Vec::new();
        for id in trivial_query_one_component($world, TypeId::of::<RefCell<$ty>>()) {
            let r = $world.get_component_by_entityid::<$ty>(id);
            let s = ::quicksilver::reflections_ref::reflect_ref(&*r).to_json();
            buffer.push(EntityComponent(id.0, s));
        }
        $state
            .components
            .insert(type_name::<$ty>().to_string(), buffer);
    };
    (@comp $world:ident $state:ident $ty:tt) => {};
    (Components($($components:tt $([$persist_comp:tt])?),*),
     Relations($($relations:tt $(($flags:expr))? $([$persist_rel:tt])?),*)) => {
        pub fn save_world(world: &World) -> String {
            let mut state = SerializedState::default();
            $(generate_save!(@comp world state $components $($persist_comp)?);)*
            $(generate_save!(@rel world state $relations $($persist_rel)?);)*
            ::quicksilver::reflections_ref::reflect_ref(&state).to_json()
        }
    };
}

macro_rules! generate_load {
    (@rel ($world:expr) $var:ident $pairs:ident $ty:tt persist) => {
        if $var == type_name::<$ty>() {
            for OriginTarget(origin, target) in $pairs {
                let a = $world.ensure_alive(EntityId(*origin));
                let b = $world.ensure_alive(EntityId(*target));
                $world.add_relation::<$ty>(a, b);
            }
            continue;
        }
    };
    (@rel ($world:expr) $var:ident $payloads:ident $ty:tt) => {};
    (@comp ($world:expr) $var:ident $payloads:ident $ty:tt persist) => {
        if $var == type_name::<$ty>() {
            for EntityComponent(entity_id, payload) in $payloads {
                let val = ::quicksilver::json::from_json::<$ty>(payload);
                let e = $world.ensure_alive(EntityId(*entity_id));
                $world.add_component(e, val);
            }
            continue;
        }
    };
    (@comp ($world:expr) $var:ident $payloads:ident $ty:tt) => {};
    (Components($($components:tt $([$persist_comp:tt])?),*),
     Relations($($relations:tt $(($flags:expr))? $([$persist_rel:tt])?),*)) => {
        #[allow(unused)]
        pub fn load_world(s: &str) -> World {
            let mut world = World::new();
            register_components(&mut world);
            let state: SerializedState = ::quicksilver::json::from_json(s);

            for (ty, payloads) in &state.components {
                let var = ty.as_str();
                $(generate_load!(@comp (&mut world) var payloads $components $($persist_comp)?);)*
                panic!("Unknown component type: {var}");
            }

            for (ty, pairs) in &state.relations {
                let var = ty.as_str();
                $(generate_load!(@rel (&mut world) var pairs $relations $($persist_rel)?);)*
                panic!("Unknown relationship type: {var}");
            }

            world
        }
    };
}

macro_rules! ecs_types {
    ($($tokens:tt)+) => {
        generate_register!($($tokens)+);
        generate_re_register!($($tokens)+);
        generate_save!($($tokens)+);
        generate_load!($($tokens)+);
    }
}

pub(crate) use ecs_types;
pub(crate) use generate_load;
pub(crate) use generate_re_register;
pub(crate) use generate_register;
pub(crate) use generate_save;
