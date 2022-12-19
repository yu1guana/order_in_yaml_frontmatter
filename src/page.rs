// Copyright (c) 2022 Yuichi Ishida

use anyhow::{bail, Context, Result};
use getset::{Getters, MutGetters, Setters};
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use yaml_rust::{Yaml, YamlEmitter};

/// ファイルのFrontMatterに関する情報を保持する
///
/// valueの値が先に変更され、別の関数によりyamlに反映させる。
#[derive(Debug, Getters, MutGetters, Setters)]
#[getset(get = "pub")]
pub struct Page {
    /// ファイル
    #[getset(get_mut, set)]
    path: PathBuf,

    /// FrontMatter
    #[getset(get_mut, set)]
    yaml: Yaml,

    /// FrontMatterのkeyの値
    #[getset(get_mut, set)]
    value: Option<i64>,

    /// 元々のFrontMatterのkeyの値
    value_old: Option<i64>,

    /// FrontMatterのtitleの値
    title: Option<String>,
}

/// FrontMatterを持つファイルのリスト
///
/// コンストラクタではFrontMatterのkeyで指定された変数の昇順に格納する。
/// 値を持たない場合は最後に回す。
/// その後の操作でもvalueの値は常に昇順になるようにする。
/// 途中の操作ではNoneが途中に挟まっても良い。
#[derive(Debug, Getters)]
pub struct PageList {
    page_list: Vec<Page>,

    /// FrontMatterの変数名
    #[getset(get)]
    key: String,
}

pub enum SwapDirection {
    Prev,
    Next,
}

#[derive(Debug, thiserror::Error)]
enum PageError {
    #[error("failed to get front matter: {0}")]
    NoFrontMatter(PathBuf),
    #[error("failed to get an integer : {0}")]
    NoIntegerKey(PathBuf),
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
            _ => return Err(PageError::NoIntegerKey(path.to_owned())),
        };
        let title = if let Yaml::String(x) = &yaml["title"] {
            Some(x.to_owned())
        } else {
            None
        };
        Ok(Self {
            path: path.to_owned(),
            yaml,
            value,
            value_old: value,
            title,
        })
    }
    fn substitute_value(&mut self, key: &str) {
        match &mut self.yaml {
            Yaml::Hash(hash) => {
                if let Some(value) = self.value {
                    hash.insert(Yaml::String(key.to_owned()), Yaml::Integer(value));
                } else {
                    hash.remove(&Yaml::String(key.to_owned()));
                }
            }
            _ => panic!(),
        }
    }
    fn overwrite_frontmatter(&mut self) -> Result<()> {
        if self.value != self.value_old {
            let mut new_file_content = String::new();
            let mut emitter = YamlEmitter::new(&mut new_file_content);
            emitter.dump(&self.yaml)?;
            writeln!(new_file_content, "\n---")?;
            let tempfile = NamedTempFile::new()?;
            let buf_reader = BufReader::new(File::open(&self.path)?);
            let mut end_yaml = false;
            for line_result in buf_reader.lines().skip(1) {
                let line = line_result?;
                if end_yaml {
                    writeln!(new_file_content, "{}", line)?;
                } else if line == "---" {
                    end_yaml = true
                }
            }
            fs::write(&tempfile, new_file_content)?;
            fs::copy(tempfile, &self.path)?;
        }
        Ok(())
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
        let page_list = Self {
            page_list: Vec::new(),
            key: key.to_owned(),
        };
        let mut page_list = page_list.append_page_list(target_dir, recursive)?;
        page_list.sort_and_fix();
        Ok(page_list)
    }

    /// ページリストを追加する
    ///
    /// target_dir: ディレクトリ指定
    /// recursive: ディレクトリを再起的に遡るかどうか
    fn append_page_list(self, target_dir: &Path, recursive: bool) -> Result<Self> {
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
                match Page::try_new(&path, &page_list.key) {
                    Ok(page) => page_list.push(page),
                    Err(PageError::NoFrontMatter(_)) => continue,
                    Err(err) => return Err(err.into()),
                }
            } else if recursive && path.is_dir() {
                page_list = page_list.append_page_list(&path, recursive)?;
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

    /// valueに値があれば外し、そうでなければ代入する
    pub fn toggle_value(&mut self, idx: usize) -> Result<()> {
        let mut pre_value = 0;
        let unset;
        if let Some(page) = self.get_mut(idx) {
            if page.value().is_some() {
                unset = true;
                page.set_value(None);
            } else {
                unset = false;
                for pre_page in self.iter().take(idx) {
                    if let Some(x) = pre_page.value() {
                        pre_value = *x;
                    }
                }
            }
        } else {
            bail!("failed to get {}-th element", idx);
        }
        if unset {
            for page in self.iter_mut().skip(idx + 1) {
                if let Some(value) = page.value() {
                    page.set_value(Some(value - 1));
                }
            }
        } else {
            self.get_mut(idx).unwrap().set_value(Some(pre_value + 1));
            for page in self.iter_mut().skip(idx + 1) {
                if let Some(value) = page.value() {
                    page.set_value(Some(value + 1));
                }
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
                        bail!("failed to get previous one");
                    }
                }
                SwapDirection::Next => {
                    if idx == self.len() - 1 {
                        bail!("failed to get next one");
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

    /// yamlにvalueを反映させる
    pub fn substitute_value(&mut self) {
        let key = self.key().clone();
        for page in self.iter_mut() {
            page.substitute_value(&key);
        }
    }

    /// Frontmatterを書き換える
    pub fn overwrite_frontmatter(&mut self) -> Result<()> {
        for page in self.iter_mut() {
            page.overwrite_frontmatter()?;
        }
        Ok(())
    }
}
