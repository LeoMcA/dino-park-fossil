use crate::send::resize::Avatars;
use crate::storage::loader::Loader;
use crate::storage::saver::Saver;
use failure::Error;
use futures::future;
use futures::future::Either;
use futures::Future;
use std::sync::Arc;

const RAW: &str = "raw";
const LARGE: &str = "264";
const MEDIUM: &str = "100";
const SMALL: &str = "40";

pub fn delete(
    name: &str,
    bucket: &str,
    saver: &Arc<impl Saver>,
) -> impl Future<Item = (), Error = Error> {
    Future::join3(
        saver.delete(name, LARGE, bucket),
        saver.delete(name, MEDIUM, bucket),
        saver.delete(name, SMALL, bucket),
    )
    .map(|_| ())
}

pub fn save(
    avatars: Avatars,
    name: &str,
    bucket: &str,
    saver: &Arc<impl Saver>,
) -> impl Future<Item = (), Error = Error> {
    let Avatars {
        raw,
        x264,
        x100,
        x40,
    } = avatars;
    Future::join4(
        saver.save(name, RAW, bucket, raw),
        saver.save(name, LARGE, bucket, x264),
        saver.save(name, MEDIUM, bucket, x100),
        saver.save(name, SMALL, bucket, x40),
    )
    .map(|_| ())
}

pub fn rename(
    old_name: &str,
    new_name: &str,
    bucket: &str,
    saver: &Arc<impl Saver>,
    loader: &Arc<impl Loader>,
) -> impl Future<Item = (), Error = Error> {
    if old_name == new_name {
        return Either::A(future::ok(()));
    }
    Either::B(
        Future::join3(
            rename_one(old_name, new_name, LARGE, bucket, saver, loader),
            rename_one(old_name, new_name, MEDIUM, bucket, saver, loader),
            rename_one(old_name, new_name, SMALL, bucket, saver, loader),
        )
        .map(|_| ()),
    )
}

fn rename_one(
    old_name: &str,
    new_name: &str,
    size: &str,
    bucket: &str,
    saver: &Arc<impl Saver>,
    loader: &Arc<impl Loader>,
) -> impl Future<Item = (), Error = Error> {
    let new_name = new_name.to_owned();
    let old_name = old_name.to_owned();
    let saver_bucket = bucket.to_owned();
    let delete_bucket = bucket.to_owned();
    let saver_size = size.to_owned();
    let delete_size = size.to_owned();
    let saver_saver = Arc::clone(saver);
    let delete_saver = Arc::clone(saver);
    loader
        .load(&old_name, &size, &bucket)
        .and_then(move |buf| saver_saver.save(&new_name, &saver_size, &saver_bucket, buf))
        .and_then(move |_| delete_saver.delete(&old_name, &delete_size, &delete_bucket))
}
