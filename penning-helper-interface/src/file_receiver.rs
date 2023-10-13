use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FileReceiverSource {
    TurfList,
    SepaSaveLoc,
}

impl FileReceiverSource {
    pub fn extensions(&self) -> &[(&str, &[&str])] {
        match self {
            FileReceiverSource::TurfList => &[("Turflist", &["xlsx", "xls", "csv"])],
            FileReceiverSource::SepaSaveLoc => &[("SEPA", &["xml"])],
        }
    }

    pub fn should_save(&self) -> bool {
        match self {
            FileReceiverSource::TurfList => false,
            FileReceiverSource::SepaSaveLoc => true,
        }
    }
}

#[derive(Debug, Default)]
pub struct FileReceievers {
    receivers: HashMap<FileReceiverSource, FileReceiver>,
    received: HashSet<FileReceiverSource>,
}

impl FileReceievers {
    pub fn new_receiver(&mut self, source: FileReceiverSource) {
        println!("Adding new receiver: {:?}", source);
        self.received.remove(&source);
        let extensions = source.extensions();
        self.receivers.insert(
            source,
            FileReceiver::recv_or_save(extensions, source.should_save()),
        );
    }

    pub fn get_receiver(&self, source: FileReceiverSource) -> Option<&FileReceiver> {
        self.receivers.get(&source)
    }

    pub fn remove_receiver(&mut self, source: FileReceiverSource) {
        self.receivers.remove(&source);
    }

    pub fn receive_all(&mut self) {
        for (p, receiver) in self.receivers.iter_mut() {
            if self.received.contains(p) {
                continue;
            }
            if !matches!(receiver.try_recv(), FileReceiverResult::Waiting) {
                self.received.insert(*p);
            }
        }
    }
}

#[derive(Debug)]
pub struct FileReceiver {
    receiver: Receiver<PathBuf>,
    file: Option<PathBuf>,
    has_received: bool,
}

pub enum FileReceiverResult<'p> {
    File(&'p Path),
    NoFile,
    Waiting,
}

impl FileReceiver {
    pub fn recv_or_save(extensions: &[(&str, &[&str])], save: bool) -> Self {
        let (s, receiver) = channel();
        let mut dialog = rfd::FileDialog::new();
        for (name, exts) in extensions {
            dialog = dialog.add_filter(name, exts);
        }
        thread::spawn(move || {
            let r = if save {
                dialog.save_file()
            } else {
                dialog.pick_file()
            };
            if let Some(res) = r {
                s.send(res).unwrap();
            }
        });
        Self {
            receiver,
            file: None,
            has_received: false,
        }
    }

    pub fn get_file(&self) -> FileReceiverResult {
        if self.has_received {
            if let Some(f) = &self.file {
                FileReceiverResult::File(f)
            } else {
                FileReceiverResult::NoFile
            }
        } else {
            FileReceiverResult::Waiting
        }
    }

    pub fn try_recv(&mut self) -> FileReceiverResult {
        if self.has_received {
            return if let Some(f) = &self.file {
                FileReceiverResult::File(f)
            } else {
                FileReceiverResult::NoFile
            };
        }

        match self.receiver.try_recv() {
            Ok(p) => {
                self.has_received = true;
                self.file = Some(p);
                self.get_file()
            }
            Err(TryRecvError::Empty) => FileReceiverResult::Waiting,
            Err(TryRecvError::Disconnected) => {
                self.has_received = true;
                FileReceiverResult::NoFile
            }
        }
    }
}
