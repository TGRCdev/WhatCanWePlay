use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::{PyTuple, PyList};

pub mod conversions {
    use pyo3::prelude::*;
    use pyo3::types::PyDict;
    use pyo3::conversion::ToPyObject;

    use crate::igdb;
    impl IntoPy<PyObject> for igdb::Token {
        fn into_py(self, py: Python) -> PyObject {
            let obj = PyDict::new(py);
            obj.set_item("access_token", self.access_token).unwrap();
            obj.set_item("expires_in", self.expires_in).unwrap();
            return obj.into();
        }
    }

    impl IntoPy<PyObject> for igdb::GameInfo {
        fn into_py(self, py: Python) -> PyObject {
            let obj = PyDict::new(py);
            obj.set_item("steam_id", self.steam_id).unwrap();
            obj.set_item("igdb_id", self.igdb_id).unwrap();
            obj.set_item("name", self.name).unwrap();
            obj.set_item("supported_players", self.supported_players).unwrap();
            obj.set_item("cover_id", self.cover_id).unwrap();
            obj.set_item("has_multiplayer", self.has_multiplayer).unwrap();

            return obj.into();
        }
    }

    impl ToPyObject for igdb::GameInfo {
        fn to_object(&self, py: Python) -> PyObject {
            let obj = PyDict::new(py);
            obj.set_item("steam_id", self.steam_id).unwrap();
            obj.set_item("igdb_id", self.igdb_id).unwrap();
            obj.set_item("name", self.name.clone()).unwrap();
            obj.set_item("supported_players", self.supported_players).unwrap();
            obj.set_item("cover_id", self.cover_id.clone()).unwrap();
            obj.set_item("has_multiplayer", self.has_multiplayer).unwrap();

            return obj.into();
        }
    }

    use crate::steam;
    impl IntoPy<PyObject> for steam::SteamUser {
        fn into_py(self, py: Python) -> PyObject {
            let obj = PyDict::new(py);
            obj.set_item("steam_id", self.steam_id).unwrap();
            obj.set_item("screen_name", self.screen_name).unwrap();
            obj.set_item("avatar_thumb", self.avatar_thumb).unwrap();
            obj.set_item("avatar", self.avatar).unwrap();
            obj.set_item("visibility", self.visibility).unwrap();
            obj.set_item("online", self.online).unwrap();

            return obj.into();
        }
    }

    impl ToPyObject for steam::SteamUser {
        fn to_object(&self, py: Python) -> PyObject {
            let obj = PyDict::new(py);
            obj.set_item("steam_id", self.steam_id).unwrap();
            obj.set_item("screen_name", self.screen_name.clone()).unwrap();
            obj.set_item("avatar_thumb", self.avatar_thumb.clone()).unwrap();
            obj.set_item("avatar", self.avatar.clone()).unwrap();
            obj.set_item("visibility", self.visibility).unwrap();
            obj.set_item("online", self.online).unwrap();

            return obj.into();
        }
    }
}

use crate::igdb;

pub mod igdb_exceptions {
    use pyo3::create_exception;
    use pyo3::exceptions::{PyException};
    use pyo3::PyErr;

    use crate::errors::IGDBError;

    create_exception!(igdb, IGDBException, PyException);

    create_exception!(igdb, UnknownErrorException, IGDBException);
    create_exception!(igdb, ServerErrorException, IGDBException);
    create_exception!(igdb, BadClientException, IGDBException);
    create_exception!(igdb, BadResponseException, IGDBException);
    create_exception!(igdb, BadSecretException, IGDBException);
    create_exception!(igdb, BadTokenException, IGDBException);
    create_exception!(igdb, BadAuthException, IGDBException);

    impl From<IGDBError> for PyErr {
        fn from(e: IGDBError) -> PyErr {
            match e {
                IGDBError::ServerError => ServerErrorException::new_err(e.to_string()),
                IGDBError::BadResponse => BadResponseException::new_err(e.to_string()),
                IGDBError::BadClient => BadClientException::new_err(e.to_string()),
                IGDBError::BadSecret => BadSecretException::new_err(e.to_string()),
                IGDBError::BadToken => BadTokenException::new_err(e.to_string()),
                IGDBError::BadAuth => BadAuthException::new_err(e.to_string()),
                _ => UnknownErrorException::new_err(e.to_string()),
            }
        }
    }
}

#[pyfunction]
pub fn fetch_twitch_token(_py: Python, client_id: &str, secret: &str) -> PyResult<igdb::Token> {
    let token = igdb::get_twitch_token(client_id, secret);
    match token {
        Ok(token) => return Ok(token),
        Err(e) => return Err(e.into())
    }
}

#[pyfunction]
pub fn get_steam_game_info(_py: Python, client_id: &str, bearer_token: &str, appids: Vec<u64>) -> PyResult<PyObject> {
    let result = igdb::get_steam_game_info(client_id, bearer_token, &appids);
    
    match result {
        Err(e) => {
            return Err(e.into());
        },
        Ok((games, not_found)) => {
            let games = PyList::new(_py, games);
            let not_found = PyList::new(_py, &not_found);
            let tuple : Vec<PyObject> = vec!(games.into(), not_found.into());
            return Ok(PyTuple::new(_py, tuple).into());
        }
    }
}

macro_rules! expose_exception {
    ($py:expr, $m:expr, $exc:ty) => {
        $m.add(stringify!($exc), $py.get_type::<$exc>());
    }
}

fn igdb_mod(py: &Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fetch_twitch_token, m)?)?;
    m.add_function(wrap_pyfunction!(get_steam_game_info, m)?)?;

    use igdb_exceptions::*;

    // Exceptions
    expose_exception!(py, m, IGDBException)?;
    expose_exception!(py, m, UnknownErrorException)?;
    expose_exception!(py, m, ServerErrorException)?;
    expose_exception!(py, m, BadClientException)?;
    expose_exception!(py, m, BadResponseException)?;
    expose_exception!(py, m, BadSecretException)?;
    expose_exception!(py, m, BadTokenException)?;
    expose_exception!(py, m, BadAuthException)?;

    return Ok(());
}

use crate::steam;

pub mod steam_exceptions {
    use pyo3::create_exception;
    use pyo3::exceptions::{PyException};
    use pyo3::PyErr;

    use crate::errors::SteamError;

    create_exception!(steam, SteamException, PyException);

    create_exception!(steam, UnknownErrorException, SteamException);
    create_exception!(steam, ServerErrorException, SteamException);
    create_exception!(steam, BadResponseException, SteamException);
    create_exception!(steam, BadWebkeyException, SteamException);
    create_exception!(steam, GamesListPrivateException, SteamException);
    create_exception!(steam, GamesListEmptyException, SteamException);
    create_exception!(steam, FriendListPrivateException, SteamException);

    impl From<SteamError> for PyErr {
        fn from(e: SteamError) -> PyErr {
            match e {
                SteamError::ServerError => ServerErrorException::new_err(e.to_string()),
                SteamError::BadResponse => BadResponseException::new_err(e.to_string()),
                SteamError::BadWebkey => BadWebkeyException::new_err(e.to_string()),
                SteamError::FriendListPrivate => FriendListPrivateException::new_err(e.to_string()),
                SteamError::GamesListPrivate(steamid) => GamesListPrivateException::new_err((e.to_string(), steamid)),
                SteamError::GamesListEmpty(steamid) => GamesListEmptyException::new_err((e.to_string(), steamid)),
                _ => UnknownErrorException::new_err(e.to_string()),
            }
        }
    }
}

#[pyfunction]
pub fn get_steam_users_info(_py: Python, webkey: &str, steamids: Vec<u64>) -> PyResult<PyObject> {
    let result = steam::get_steam_users_info(webkey, &steamids);

    match result {
        Err(e) => {
            return Err(e.into());
        },
        Ok(users) => {
            return Ok(PyList::new(_py, users).into());
        },
    }
}

#[pyfunction]
pub fn get_owned_steam_games(_py: Python, webkey: &str, steamid: u64) -> PyResult<PyObject> {
    let result = steam::get_owned_steam_games(webkey, steamid);

    match result {
        Err(e) => {
            return Err(e.into());
        },
        Ok(game_ids) => {
            return Ok(PyList::new(_py, game_ids).into());
        }
    }
}

#[pyfunction]
pub fn get_friend_list(_py: Python, webkey: &str, steamid: u64) -> PyResult<PyObject> {
    let result = steam::get_friend_list(webkey, steamid);

    match result {
        Err(e) => {
            return Err(e.into());
        },
        Ok(friends) => {
            return Ok(PyList::new(_py, friends).into());
        }
    }
}

#[pyfunction]
pub fn intersect_owned_game_ids(_py: Python, webkey: &str, steamids: Vec<u64>) -> PyResult<PyObject> {
    let result = steam::intersect_owned_game_ids(webkey, &steamids);

    match result {
        Err(e) => {
            return Err(e.into());
        },
        Ok(appids) => {
            return Ok(PyList::new(_py, appids).into());
        }
    }
}

fn steam_mod(py: &Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_steam_users_info, m)?)?;
    m.add_function(wrap_pyfunction!(get_owned_steam_games, m)?)?;
    m.add_function(wrap_pyfunction!(get_friend_list, m)?)?;
    m.add_function(wrap_pyfunction!(intersect_owned_game_ids, m)?)?;

    use steam_exceptions::*;

    // Exceptions
    expose_exception!(py, m, SteamException)?;
    expose_exception!(py, m, UnknownErrorException)?;
    expose_exception!(py, m, ServerErrorException)?;
    expose_exception!(py, m, BadResponseException)?;
    expose_exception!(py, m, BadWebkeyException)?;
    expose_exception!(py, m, FriendListPrivateException)?;
    expose_exception!(py, m, GamesListEmptyException)?;
    expose_exception!(py, m, GamesListPrivateException)?;

    return Ok(());
}

use crate::wcwp;

pub mod wcwp_exceptions {
    use pyo3::PyErr;

    use crate::errors::WCWPError;

    impl From<WCWPError> for PyErr {
        fn from(e: WCWPError) -> PyErr {
            match e {
                WCWPError::IGDBError(e) => return e.into(),
                WCWPError::SteamError(e) => return e.into(),
            }
        }
    }
}

#[pyfunction]
pub fn intersect_owned_games(_py: Python, webkey: &str, igdb_id: &str, igdb_token: &str, steamids: Vec<u64>) -> PyResult<PyObject> {
    let result = wcwp::intersect_owned_games(webkey, igdb_id, igdb_token, &steamids)?;

    return Ok(PyList::new(_py, result).into());
}

#[pymodule]
fn whatcanweplay(py: Python, m: &PyModule) -> PyResult<()> {
    let submod = PyModule::new(py, "igdb")?;
    igdb_mod(&py, submod)?;
    m.add_submodule(submod)?;

    let submod = PyModule::new(py, "steam")?;
    steam_mod(&py, submod)?;
    m.add_submodule(submod)?;

    m.add_function(wrap_pyfunction!(intersect_owned_games, m)?)?;

    return Ok(());
}