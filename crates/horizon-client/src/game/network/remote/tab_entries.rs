//! The tab list's entries as the client holds them, for reads that must not change them.

use super::Remote;
use horizon_client::tab_list::TabPlayer;

impl Remote {
    /// The listed players (`ClientPacketListener.getListedOnlinePlayers`), in no order:
    /// `tab_list::sorted_players` gives the overlay's.
    pub(in crate::game) fn listed_tab_players(&self) -> Vec<TabPlayer> {
        self.listed
            .iter()
            .filter_map(|uuid| self.tab_players.get(uuid).cloned())
            .collect()
    }
}

impl Remote {
    /// The listed players' skins as the Tab overlay draws them: each player's resolved skin
    /// texture path and whether it uses the slim model, in no order. A skin still downloading
    /// reads as the profile's default (`tab_list::default_skin`), as the overlay draws it.
    pub(in crate::game) fn listed_player_skins(&self) -> Vec<(u128, String, bool)> {
        self.listed
            .iter()
            .map(|uuid| {
                let skin = self.skins.get(uuid).map_or_else(
                    || horizon_client::tab_list::default_skin(*uuid).to_owned(),
                    |skin| skin.clone(),
                );
                // The resolved skin's model is the one `update_remote_skins`
                // kept (`skin_slim`): the textures' `slim`, plus the local
                // player's launcher offline skin and signed-in account skin,
                // which `skin_textures` never holds.
                let slim = match self.skins.get(uuid) {
                    Some(_) => self.skin_slim.contains(uuid),
                    None => skin.contains("/slim/"),
                };
                (*uuid, skin, slim)
            })
            .collect()
    }
}
