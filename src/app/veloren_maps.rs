use std::{collections::BTreeMap, sync::OnceLock};

use anyhow::anyhow;
use log::info;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use yew::{html, Component, Context, Html};
use yew_router::components::Link;

use crate::app::Route;

#[derive(Deserialize, Serialize)]
enum MapKind {
    Square,
    Circle,
}

#[derive(Deserialize, Serialize)]
struct GenOpts {
    x_lg: u8,
    y_lg: u8,
    scale: f32,
    map_kind: MapKind,
    erosion_quality: f32,
}

#[derive(Deserialize, Serialize)]
pub struct MapData {
    seed: u32,
    gen_opts: GenOpts,
}

impl MapData {
    async fn get(meta: &MapMeta) -> anyhow::Result<Self> {
        let text = reqwest::get(&meta.data_url).await?.text().await?;

        info!("DATA: {text}");
        let res = ron::from_str(&text)?;

        Ok(res)
    }

    fn html(&self) -> Html {
        if let Ok(data) = ron::ser::to_string_pretty(self, PrettyConfig::new()) {
            html! {
                <div class="relative max-w-2xl mx-auto text-left">
                    <div class="bg-gray-900 text-white p-4 rounded-md">
                        <div class="overflow-x-auto">
                            <pre id="code" class="text-gray-300">
                                <code>{data}</code>
                            </pre>
                        </div>
                    </div>
                </div>
            }
        } else {
            html!()
        }
    }
}

#[derive(Default)]
struct MapMeta {
    seed: u32,
    data_url: String,
    image_url: String,
}

impl MapMeta {
    fn html(&self) -> Html {
        html! {
            <Link<Route> to={Route::VelorenMap { seed: self.seed }}>
                <div class = "rounded-md bg-zinc-800 hover:bg-zinc-600 px-1 py-1">
                    <img src = { self.image_url.clone() } width="256" height="256" class="px-2 py-2"/>
                    <h0 class="bold">{self.seed.to_string()}</h0>
                </div>
            </Link<Route>>
        }
    }
}

pub struct Maps {
    maps: BTreeMap<u32, MapMeta>,
}

impl Maps {
    async fn get() -> anyhow::Result<&'static Self> {
        const REPO_CONTENT: &str = "https://api.github.com/repos/IsseW/veloren_maps/contents/";
        static CACHED: OnceLock<Maps> = OnceLock::new();
        match CACHED.get() {
            Some(maps) => Ok(maps),
            None => {
                let res: String = reqwest::get(REPO_CONTENT).await?.text().await?;

                #[derive(Deserialize)]
                struct GhFile {
                    name: String,
                    download_url: String,
                }
                let files = serde_json::from_str::<Vec<GhFile>>(&res)?;

                let mut maps = BTreeMap::new();

                for mut file in files {
                    if let Some((seed, ty)) = file
                        .name
                        .split_once('.')
                        .and_then(|(s, f)| Some((s.parse().ok()?, f)))
                    {
                        let meta = maps.entry(seed).or_insert(MapMeta::default());
                        meta.seed = seed;
                        match ty {
                            "ron" => {
                                meta.data_url = file.download_url;
                            }
                            "png" => {
                                file.download_url.insert_str(34, "/media/");
                                file.download_url.replace_range(8..11, "media");
                                meta.image_url = file.download_url;
                            }
                            _ => {}
                        }
                    }
                }

                maps.retain(|_, d| !d.data_url.is_empty() && !d.image_url.is_empty());

                let _ = CACHED.set(Self { maps });
                CACHED.get().ok_or(anyhow!("What!?"))
            }
        }
    }
}

pub struct VelorenMaps {
    maps: Option<anyhow::Result<&'static Maps>>,
}

impl Component for VelorenMaps {
    type Message = anyhow::Result<&'static Maps>;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { maps: None }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.maps {
            Some(Ok(maps)) => {
                html! {
                    <div class="text-center">
                        <ul class="flex flex-wrap justify-center">
                            {
                                maps.maps.values().map(|meta| meta.html()).collect::<Html>()
                            }
                        </ul>
                    </div>
                }
            }
            Some(Err(e)) => {
                html! { {format!("Failed to get maps from github: {e}")} }
            }
            None => {
                ctx.link().send_future(Maps::get());
                html! { "Loading" }
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.maps = Some(msg);
        true
    }
}

pub struct VelorenMap {
    maps: Option<anyhow::Result<&'static Maps>>,
    data: Option<anyhow::Result<MapData>>,
}

#[derive(yew::Properties, PartialEq, Eq)]
pub struct MapProperties {
    pub seed: u32,
}

pub enum Message {
    Maps(anyhow::Result<&'static Maps>),
    Data(anyhow::Result<MapData>),
}

impl Component for VelorenMap {
    type Message = Message;

    type Properties = MapProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            maps: None,
            data: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.maps {
            Some(Ok(maps)) => {
                if let Some(map) = maps.maps.get(&ctx.props().seed) {
                    let data = match &self.data {
                        Some(Ok(data)) => data.html(),
                        Some(Err(e)) => {
                            html! { {format!("Failed to get data from github: {e}")} }
                        }
                        None => {
                            ctx.link()
                                .send_future(async { Message::Data(MapData::get(map).await) });
                            html! { "Loading" }
                        }
                    };
                    html! {
                        <>
                            <div class="h-3/4 justify-center">
                                <img src = { map.image_url.clone() } class="object-contain h-full w-full"/>
                            </div>
                            <div class = "text-center">
                                {data}
                            </div>
                        </>
                    }
                } else {
                    html! {
                        <span>
                            {"Map doesn't exist. "}
                            <Link<Route> to={Route::VelorenMaps}>
                                {"Back to maps"}
                            </Link<Route>>
                        </span>
                    }
                }
            }
            Some(Err(e)) => {
                html! { {format!("Failed to get maps from github: {e}")} }
            }
            None => {
                ctx.link()
                    .send_future(async { Message::Maps(Maps::get().await) });
                html! { "Loading" }
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Maps(m) => self.maps = Some(m),
            Message::Data(d) => self.data = Some(d),
        }
        true
    }
}
