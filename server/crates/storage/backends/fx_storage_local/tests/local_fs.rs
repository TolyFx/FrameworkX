//! LocalFs 后端契约测试。
//!
//! 职责:验证 put/get/delete/exists/url_for 的语义一致性(后端契约)。
//! 边界:用 tempdir 隔离,不依赖真实路径、权限或外部服务。
//! 约束:仅测 StorageBackend trait 行为,不测可复用编排层(见 fx_storage/tests)。

use fx_storage_core::StorageBackend;
use fx_storage_local::LocalFs;

fn backend(dir: &tempfile::TempDir) -> LocalFs {
    LocalFs::new(dir.path().to_path_buf(), "/uploads".into())
}

#[tokio::test]
async fn put_get_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let fs = backend(&dir);
    fs.put("a/b.txt", b"hello").await.unwrap();
    let got = fs.get("a/b.txt").await.unwrap();
    assert_eq!(got, b"hello");
}

#[tokio::test]
async fn put_creates_nested_parent_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let fs = backend(&dir);
    fs.put("deep/nested/dir/f.txt", b"x").await.unwrap();
    assert!(fs.exists("deep/nested/dir/f.txt").await.unwrap());
}

#[tokio::test]
async fn exists_false_for_missing() {
    let dir = tempfile::tempdir().unwrap();
    let fs = backend(&dir);
    assert_eq!(fs.exists("nope.txt").await.unwrap(), false);
}

#[tokio::test]
async fn delete_removes_file() {
    let dir = tempfile::tempdir().unwrap();
    let fs = backend(&dir);
    fs.put("f.txt", b"x").await.unwrap();
    fs.delete("f.txt").await.unwrap();
    assert_eq!(fs.exists("f.txt").await.unwrap(), false);
}

#[tokio::test]
async fn url_for_prefixes_path() {
    let dir = tempfile::tempdir().unwrap();
    let fs = backend(&dir);
    assert_eq!(fs.url_for("a/b.jpg"), "/uploads/a/b.jpg");
}

#[tokio::test]
async fn get_missing_errors() {
    let dir = tempfile::tempdir().unwrap();
    let fs = backend(&dir);
    assert!(fs.get("missing").await.is_err());
}
