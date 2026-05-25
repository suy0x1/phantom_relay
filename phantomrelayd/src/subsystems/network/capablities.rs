#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum NetworkCapability {
    TransparentProxy,
    DNSIntercept,
    QUICBlocking,
    LocalhostBypass,
}