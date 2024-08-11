use console::style;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use termion::terminal_size;

#[derive(Debug, Serialize, Deserialize)]
struct PackageLinks {
    npm: String,
    homepage: Option<String>,
    repository: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NpmPackage {
    pub name: String,
    description: Option<String>,
    version: String,
    keywords: Option<Vec<String>>,
    date: String,
    links: PackageLinks,
}

#[derive(Debug, Serialize, Deserialize)]
struct NpmPackageObject {
    package: NpmPackage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NpmRegistryResponse {
    objects: Vec<NpmPackageObject>,
}

#[derive(Debug)]
pub struct Choice {
    pub title: String,
    pub value: NpmPackage,
}
impl Display for Choice {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

fn format_package_with_url(name_version: &str, url: &str, terminal_columns: u16) -> String {
    format!(
        "{:<width$} {}",
        name_version,
        url,
        width = terminal_columns as usize - url.len()
    )
}

pub async fn fetch_npm_packages(pattern: &str) -> Result<Vec<Choice>, Box<dyn Error>> {
    let registry_link = format!(
        "https://registry.npmjs.com/-/v1/search?text={}&size=35",
        pattern
    );

    let (terminal_columns, _) = terminal_size().unwrap_or((80, 0));

    let resp = reqwest::get(&registry_link)
        .await?
        .json::<NpmRegistryResponse>()
        .await?;

    let choices: Vec<Choice> = resp
        .objects
        .into_iter()
        .map(|obj| {
            let pkg = obj.package;

            let repository_url = pkg
                .links
                .repository
                .as_ref()
                .unwrap_or_else(|| pkg.links.homepage.as_ref().unwrap_or(&pkg.links.npm));

            let title = format_package_with_url(
                &format!(
                    "{:width$} v{}",
                    pkg.name,
                    style(pkg.version.clone()).blue(),
                    width = 30
                ),
                &repository_url,
                terminal_columns,
            );
            Choice { title, value: pkg }
        })
        .collect();

    Ok(choices)
}
