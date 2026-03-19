/// save_worker.rs — Background thread that serialises and writes workspace files.
///
/// Decouples the main (UI) thread from disk I/O entirely:
///
///   1.  The UI thread calls `SaveWorker::queue()` with a cloned `WorkspaceFile`.
///   2.  The worker thread receives it, drains any newer requests that arrived
///       while it was busy (latest-wins), then does:
///         a.  MessagePack serialisation  (CPU-bound, runs off main thread)
///         b.  Atomic write: stream to `<path>.cards.tmp`, then `rename` over the
///             real file — prevents partial/corrupt files on crash or power loss.
///
/// Because `CardData.image_data` is now `Option<Arc<Vec<u8>>>`, moving a
/// `WorkspaceFile` into the channel is O(1) — the Arc just bumps a ref-count.
/// The actual byte traversal (serialise) happens entirely on this worker thread.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

use crate::workspace::WorkspaceFile;

struct SaveRequest {
    workspace: WorkspaceFile,
    path:      PathBuf,
}

pub struct SaveWorker {
    tx: Sender<SaveRequest>,
}

impl SaveWorker {
    /// Spawn the background worker thread and return a handle.
    pub fn spawn() -> Self {
        let (tx, rx): (Sender<SaveRequest>, Receiver<SaveRequest>) = mpsc::channel();

        thread::Builder::new()
            .name("cards-save".into())
            .spawn(move || worker_loop(rx))
            .expect("failed to spawn save worker thread");

        Self { tx }
    }

    /// Queue a save.  Cheap: just moves the `WorkspaceFile` into the channel.
    /// If the worker is still busy with a previous write, the newer request
    /// replaces it (latest-wins drain in `worker_loop`).
    pub fn queue(&self, workspace: WorkspaceFile, path: PathBuf) {
        // Ignore send errors — they only happen if the thread panicked.
        let _ = self.tx.send(SaveRequest { workspace, path });
    }
}

fn worker_loop(rx: Receiver<SaveRequest>) {
    while let Ok(first) = rx.recv() {
        // Drain any queued-up requests that arrived while we were sleeping;
        // only the latest one matters.
        let mut latest = first;
        loop {
            match rx.try_recv() {
                Ok(newer)                   => { latest = newer; }
                Err(TryRecvError::Empty)    => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        if let Err(e) = write_atomic(latest) {
            eprintln!("[save] write failed: {e}");
        }
    }
}

fn write_atomic(req: SaveRequest) -> Result<(), String> {
    // Write to a sibling temp file first, then rename — atomic on Linux/macOS,
    // best-effort on Windows (rename overwrites but is not transactional).
    let tmp = req.path.with_extension("cards.tmp");

    req.workspace
        .save(&tmp)
        .map_err(|e| e.to_string())?;

    std::fs::rename(&tmp, &req.path).map_err(|e| {
        // Clean up the temp file if rename fails.
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })
}
