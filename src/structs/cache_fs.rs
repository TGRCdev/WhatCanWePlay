use std::{
    path::{ PathBuf, Path },
};
use rocket::{
    http::{ Method, Header },
    fs::{ Options, NamedFile },
    route::{ Handler, Route, Outcome },
    Request, Data,
    response::Redirect,
};
use chrono::{
    prelude::*,
    DurationRound,
    Duration,
};

#[derive(Debug, Clone)]
pub struct CachedFileServer {
    root: PathBuf,
    options: Options,
    rank: isize,
}

// A lot of this code is lifted directly from Rocket's FileServer
impl CachedFileServer {
    const DEFAULT_RANK: isize = 10;

    pub fn new<P: AsRef<Path>>(path: P, options: Options) -> Self {
        use rocket::yansi::Paint;

        let path = path.as_ref();
        if !path.is_dir() {
            let path = path.display();
            error!("FileServer path '{}' is not a directory.", Paint::white(path));
            warn_!("Aborting early to prevent inevitable handler failure.");
            panic!("bad FileServer path: refusing to continue");
        }

        CachedFileServer { root: path.into(), options, rank: Self::DEFAULT_RANK }
    }

    pub fn rank(mut self, rank: isize) -> Self {
        self.rank = rank;
        self
    }

    pub fn from<P: AsRef<Path>>(path: P) -> Self {
        CachedFileServer::new(path, Options::default())
    }
}

impl Into<Vec<Route>> for CachedFileServer {
    fn into(self) -> Vec<Route> {
        let source = figment::Source::File(self.root.clone());
        let mut route = Route::ranked(self.rank, Method::Get, "/<path..>", self);
        route.name = Some(format!("CachedFileServer: {}/", source).into());
        vec![route]
    }
}

#[derive(Responder)]
#[response(status = 304)]
pub struct CacheHitResponder {
    response: (),
    last_modified: Header<'static>,
}

#[derive(Responder)]
#[response(status = 200)]
pub struct CacheMissResponder {
    inner: NamedFile,
    last_modified: Header<'static>,
}

impl CacheMissResponder {
    pub fn new(file: NamedFile, time: DateTime<Utc>) -> Self {
        CacheMissResponder {
            inner: file,
            last_modified: Header {
                name: "Last-Modified".into(),
                value: time.to_rfc2822().into(),
            }
        }
    }
}

impl From<DateTime<Utc>> for CacheHitResponder {
    fn from(time: DateTime<Utc>) -> Self {
        Self {
            response: (),
            last_modified: Header {
                name: "Last-Modified".into(),
                value: time.to_rfc2822().into(),
            }
        }
    }
}

#[rocket::async_trait]
impl Handler for CachedFileServer {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        use rocket::http::uri::{
            fmt::Path,
            Segments,
        };
        use rocket::http::ext::IntoOwned;

        // Get the segments as a `PathBuf`, allowing dotfiles requested.
        let options = self.options;
        let allow_dotfiles = options.contains(Options::DotFiles);
        let path = req.segments::<Segments<'_, Path>>(0..).ok()
            .and_then(|segments| segments.to_path_buf(allow_dotfiles).ok())
            .map(|path| self.root.join(path));

        match path {
            Some(p) if p.is_dir() => {
                // Normalize '/a/b/foo' to '/a/b/foo/'.
                if options.contains(Options::NormalizeDirs) && !req.uri().path().ends_with('/') {
                    let normal = req.uri().map_path(|p| format!("{}/", p))
                        .expect("adding a trailing slash to a known good path => valid path")
                        .into_owned();

                    return Outcome::from_or_forward(req, data, Redirect::permanent(normal));
                }

                if !options.contains(Options::Index) {
                    return Outcome::forward(data);
                }

                let index = NamedFile::open(p.join("index.html")).await.ok();
                Outcome::from_or_forward(req, data, index)
            },
            Some(p) => {
                let response = NamedFile::open(p).await.ok();
                match response {
                    Some(namedfile) => {
                        let file = namedfile.file();

                        if let Ok(metadata) = file.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            let modified = DateTime::<Utc>::from(modified).duration_trunc(Duration::seconds(1)).unwrap();
                            if let Some(modified_since) = req.headers().get_one("If-Modified-Since") {
                            if let Ok(modified_since) = DateTime::parse_from_rfc2822(modified_since) {
                                if modified_since >= modified {
                                    return Outcome::from(req, CacheHitResponder::from(modified));
                                }
                            }}
                            return Outcome::from(req, CacheMissResponder::new(namedfile, modified))
                        }}

                        Outcome::from_or_forward(req, data, namedfile)
                    },
                    None => Outcome::forward(data),
                }
            },
            None => Outcome::forward(data),
        }
    }
}