use crate::{igdb, steam};
use crate::errors::WCWPError;

use std::iter::FromIterator;

pub fn intersect_owned_games(webkey: &str, igdb_id: &str, igdb_token: &str, steamids: &[u64]) -> Result<Vec<igdb::GameInfo>, WCWPError>
{
    if steamids.is_empty()
    {
        return Ok(Vec::new());
    }

    let mut games_set = steam::get_owned_steam_games(webkey, steamids[0])?;

    for &id in steamids[1..].iter() {
        let next_set = steam::get_owned_steam_games(webkey, id)?;
        games_set = &games_set & &next_set; // Intersect the two sets

        if games_set.is_empty()
        { // No common owned games
            return Ok(Vec::new());
        }
    }

    let games_list = Vec::from_iter(games_set.into_iter());
    let (games_info, _) = igdb::get_steam_game_info(igdb_id, igdb_token, &games_list)?;

    return Ok(games_info);
}