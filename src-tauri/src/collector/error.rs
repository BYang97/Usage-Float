use thiserror::Error;

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("未找到 OpenCode 数据目录或数据库文件")]
    NotFound,

    #[error("打开数据库失败: {0}")]
    OpenFailed(String),

    #[error("SQL 查询失败: {0}")]
    QueryFailed(String),

    #[error("数据解析失败: {0}")]
    ParseFailed(String),

    #[error("用户未设置 auth cookie，API 功能不可用")]
    NoCookie,

    #[error("opencode.ai API 请求失败: {0}")]
    ApiError(String),

    #[error("cookie 已失效或过期")]
    Unauthorized,
}
