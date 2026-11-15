//! Connection lifecycle of the authoritative recipe property generation.
#[cfg(test)]
mod tests {
    use crate::game::{execution_role::ExecutionRole, native_entity_refs::tests::with_game};
    use horizon_protocol::{
        connection::Pipe,
        item::Stack,
        packets::recipes::{HolderSet, SlotDisplay, UpdateRecipes},
    };

    #[test]
    #[ignore = "daily"]
    fn a_received_stonecutter_list_is_the_stations_until_disconnect() {
        with_game(|game| {
            game.options.execution_role = ExecutionRole::Client;
            let (client, _server) = Pipe::pair();
            game.connect_local(client, "StationA").unwrap();
            let items = game
                .remote
                .as_ref()
                .unwrap()
                .builtin
                .get("minecraft:item")
                .unwrap()
                .clone();
            let id = |name: &str| items.id(name).unwrap();
            let own: Vec<_> = game
                .station_recipes()
                .stonecutting
                .iter()
                .map(|recipe| recipe.id.clone())
                .collect();
            assert!(!own.is_empty(), "the window's own stonecutter list");
            game.remote_recipe_properties(UpdateRecipes {
                properties: vec![],
                stonecutter: vec![
                    (
                        HolderSet::Items(vec![id("minecraft:stone")]),
                        SlotDisplay::ItemStack(Stack::new(id("minecraft:redstone"), 3)),
                    ),
                    // A result no registry names keeps its number.
                    (
                        HolderSet::Items(vec![id("minecraft:stone")]),
                        SlotDisplay::Empty,
                    ),
                ],
            });
            let received: Vec<_> = game
                .station_recipes()
                .stonecutting
                .iter()
                .map(|recipe| {
                    (
                        recipe.input.items.len(),
                        recipe.result.item.to_string(),
                        recipe.result.count,
                    )
                })
                .collect();
            assert_eq!(
                received,
                [
                    (1, "minecraft:redstone".to_owned(), 3),
                    (1, "minecraft:air".to_owned(), 0),
                ]
            );
            game.client_logging_out();
            let after: Vec<_> = game
                .station_recipes()
                .stonecutting
                .iter()
                .map(|recipe| recipe.id.clone())
                .collect();
            assert_eq!(
                after, own,
                "leaving the server restores the window's own list"
            );
        });
    }

    #[test]
    #[ignore = "daily"]
    fn disconnect_discards_properties_before_reconnect() {
        with_game(|game| {
            game.options.execution_role = ExecutionRole::Client;
            let (client, _server) = Pipe::pair();
            game.connect_local(client, "PropertiesA").unwrap();
            game.recipe_ui.server_properties.replace(UpdateRecipes {
                properties: vec![("first:input".into(), vec![7])],
                stonecutter: vec![],
            });
            // Closing a menu is not leaving the connection.
            game.recipe_ui.reset_screen();
            assert!(
                game.recipe_ui
                    .server_properties
                    .sets
                    .as_ref()
                    .unwrap()
                    .contains_key("first:input")
            );
            game.client_logging_out();
            assert!(game.recipe_ui.server_properties.sets.is_none());
            let (client, _server) = Pipe::pair();
            game.connect_local(client, "PropertiesB").unwrap();
            assert!(game.recipe_ui.server_properties.sets.is_none());
            game.recipe_ui
                .server_properties
                .replace(UpdateRecipes::default());
            assert!(
                game.recipe_ui
                    .server_properties
                    .sets
                    .as_ref()
                    .unwrap()
                    .is_empty()
            );
        });
    }
}
