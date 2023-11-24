use diesel_async::{ pooled_connection::bb8::Pool, AsyncPgConnection };
use bb8_async_memcached::MemcacheConnectionManager;

pub struct AppState {
    pub mc_pool: bb8::Pool<MemcacheConnectionManager>,
    pub pg_pool: Pool<AsyncPgConnection>,
}
