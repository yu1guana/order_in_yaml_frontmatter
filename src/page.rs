// Copyright (c) 2022 Yuichi Ishida

use anyhow::{bail, Context, Result};
use getset::{Getters, MutGetters, Setters};
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use yaml_rust::Yaml;

#[derive(Debug, Getters, MutGetters, Setters)]
#[getset(get = "pub", get_mut, set)]
pub struct Page {
    path: PathBuf,
    yaml: Yaml,
    value: Option<i64>,
}

#[derive(Debug, Default)]
pub struct PageList {
    page_list: Vec<Page>,
}

pub enum SwapDirection {
    Prev,
    Next,
}

#[derive(Debug, thiserror::Error)]
enum PageError {
    #[error("failed to get front matter: {0}")]
    NoFrontMatter(PathBuf),
    #[error("failed to get an integer `{1}`: {0}")]
    NoIntegerKey(PathBuf, String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Page {
    fn try_new(path: &Path, key: &str) -> Result<Self, PageError> {
        let yaml = frontmatter::parse(
            &fs::read_to_string(&path).map_err(|err| PageError::Other(err.into()))?,
        )
        .map_err(|err| PageError::Other(err.into()))?
        .ok_or_else(|| PageError::NoFrontMatter(path.to_owned()))?;
        let value = match &yaml[key] {
            Yaml::Integer(x) => Some(x.to_owned()),
            Yaml::BadValue | Yaml::Null => Option::None,
            _ => return Err(PageError::NoIntegerKey(path.to_owned(), key.to_owned())),
        };
        Ok(Self {
            path: path.to_owned(),
            yaml,
            value,
        })
    }
}

impl Deref for PageList {
    type Target = Vec<Page>;
    fn deref(&self) -> &Self::Target {
        &self.page_list
    }
}
impl DerefMut for PageList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.page_list
    }
}

impl PageList {
    pub fn try_new(key: &str, target_dir: &Path, recursive: bool) -> Result<Self> {
        let mut page_list = PageList::default().append_page_list(key, target_dir, recursive)?;
        page_list.sort_and_fix();
        Ok(page_list)
    }

    /// ページリストを追加する
    ///
    /// key: FrontMatterの変数を指定
    /// target_dir: ディレクトリ指定
    /// recursive: ディレクトリを再起的に遡るかどうか
    fn append_page_list(self, key: &str, target_dir: &Path, recursive: bool) -> Result<Self> {
        let mut page_list = self;
        for entry_result in target_dir
            .read_dir()
            .with_context(|| format!("faild to open {}", target_dir.display()))?
        {
            let path = entry_result?.path();
            if path.is_file()
                && (path.extension() == Some(OsStr::new("html"))
                    || path.extension() == Some(OsStr::new("md")))
            {
                match Page::try_new(&path, key) {
                    Ok(page) => page_list.push(page),
                    Err(PageError::NoFrontMatter(_)) => continue,
                    Err(err) => return Err(err.into()),
                }
            } else if recursive && path.is_dir() {
                page_list = page_list.append_page_list(key, &path, recursive)?;
            }
        }
        Ok(page_list)
    }

    /// value の値でソートし、0はじまりの連番をふる。NoneはSomeと比較すると大きい。
    fn sort_and_fix(&mut self) {
        self.sort_by(|a, b| {
            if let Some(a_value) = a.value() {
                if let Some(b_value) = b.value() {
                    a_value.cmp(b_value)
                } else {
                    Ordering::Less
                }
            } else if b.value().is_some() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        let mut current_value = 0;
        for page in self.iter_mut() {
            if page.value().is_some() {
                page.set_value(Some(current_value));
                current_value += 1;
            }
        }
    }

    /// key から値を外す
    pub fn unset(&mut self, idx: usize) -> Result<()> {
        if let Some(page) = self.get_mut(idx) {
            if page.value().is_some() {
                page.set_value(None);
            } else {
                return Ok(());
            }
        } else {
            bail!("failed to get {}-th element", idx);
        }
        for page in self.iter_mut().skip(idx + 1) {
            if let Some(value) = page.value() {
                page.set_value(Some(value - 1));
            }
        }
        Ok(())
    }

    /// key に値を入れる
    pub fn set(&mut self, idx: usize) -> Result<()> {
        let mut pre_value = 0;
        if let Some(page) = self.get(idx) {
            if page.value().is_none() {
                for pre_page in self.iter().take(idx) {
                    if let Some(x) = pre_page.value() {
                        pre_value = *x;
                    }
                }
            } else {
                return Ok(());
            }
        } else {
            bail!("failed to get {}-th element", idx);
        };
        self.get_mut(idx).unwrap().set_value(Some(pre_value + 1));
        for page in self.iter_mut().skip(idx + 1) {
            if let Some(value) = page.value() {
                page.set_value(Some(value + 1));
            }
        }
        Ok(())
    }

    /// ともにNoneでなければvalueも入れ替える。
    pub fn swap_with_value(&mut self, idx: usize, swap_direction: SwapDirection) -> Result<()> {
        if idx >= self.len() {
            bail!("failed to get {}-th element", idx);
        } else {
            match swap_direction {
                SwapDirection::Prev => {
                    if idx == 0 {
                        bail!("failed to previous one");
                    }
                }
                SwapDirection::Next => {
                    if idx == self.len() - 1 {
                        bail!("failed to next one");
                    }
                }
            }
        }
        let idx_neighbor = match swap_direction {
            SwapDirection::Prev => idx - 1,
            SwapDirection::Next => idx + 1,
        };
        if self.get(idx).unwrap().value().is_some()
            && self.get(idx_neighbor).unwrap().value().is_some()
        {
            let value_neighbor = self.get(idx_neighbor).unwrap().value().unwrap();
            let value = self.get(idx).unwrap().value().unwrap();
            self.get_mut(idx).unwrap().set_value(Some(value_neighbor));
            self.get_mut(idx_neighbor).unwrap().set_value(Some(value));
        }
        self.swap(idx, idx_neighbor);
        Ok(())
    }
}
