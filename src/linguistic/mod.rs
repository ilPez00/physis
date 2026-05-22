pub mod piraha;
pub mod sanskrit;
pub mod wenyan;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinguisticLense {
    Sanskrit,
    Piraha,
    Wenyan,
}

impl LinguisticLense {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sanskrit => "SANSKRIT",
            Self::Piraha => "PIRAHA",
            Self::Wenyan => "WENYAN",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinguisticRouter {
    pub wenyan: wenyan::WenyanFilter,
    pub piraha: piraha::PirahaFilter,
    pub sanskrit: sanskrit::SanskritEngine,
}

impl LinguisticRouter {
    pub fn new() -> Self {
        Self {
            wenyan: wenyan::WenyanFilter::new(),
            piraha: piraha::PirahaFilter::new(),
            sanskrit: sanskrit::SanskritEngine::new(),
        }
    }

    pub fn route(&self, raw_data: &str, lense: LinguisticLense) -> String {
        match lense {
            LinguisticLense::Piraha => self.piraha.filter(raw_data),
            LinguisticLense::Wenyan => self.wenyan.compress(raw_data),
            LinguisticLense::Sanskrit => self.sanskrit.dream(raw_data),
        }
    }

    pub fn route_all(&self, raw_data: &str) -> HashMap<LinguisticLense, String> {
        let mut results = HashMap::new();
        for lense in &[LinguisticLense::Wenyan, LinguisticLense::Piraha, LinguisticLense::Sanskrit] {
            results.insert(*lense, self.route(raw_data, *lense));
        }
        results
    }
}

impl Default for LinguisticRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_all_lenses() {
        let router = LinguisticRouter::new();
        let data = "ran 5k exercise";
        for lense in &[LinguisticLense::Wenyan, LinguisticLense::Piraha, LinguisticLense::Sanskrit] {
            let output = router.route(data, *lense);
            assert!(!output.is_empty(), "{} should produce output", lense.as_str());
        }
    }

    #[test]
    fn test_router_all_returns_map() {
        let router = LinguisticRouter::new();
        let data = "morning run code";
        let results = router.route_all(data);
        assert_eq!(results.len(), 3);
        assert!(results.contains_key(&LinguisticLense::Wenyan));
        assert!(results.contains_key(&LinguisticLense::Piraha));
        assert!(results.contains_key(&LinguisticLense::Sanskrit));
    }

    #[test]
    fn test_router_wenyan_produces_kanji() {
        let router = LinguisticRouter::new();
        let result = router.route("run body health", LinguisticLense::Wenyan);
        assert!(result.contains('跑') || result.contains('身') || result.contains('康'));
    }

    #[test]
    fn test_router_piraha_strips_fillers() {
        let router = LinguisticRouter::new();
        let result = router.route("I am running the code", LinguisticLense::Piraha);
        assert!(!result.contains("I"));
        assert!(!result.contains("the"));
        assert!(result.contains("running"));
        assert!(result.contains("code"));
    }

    #[test]
    fn test_router_sanskrit_expands() {
        let router = LinguisticRouter::new();
        let result = router.route("run code", LinguisticLense::Sanskrit);
        assert!(result.contains("blossoms into"));
    }

    #[test]
    fn test_lense_as_str() {
        assert_eq!(LinguisticLense::Wenyan.as_str(), "WENYAN");
        assert_eq!(LinguisticLense::Piraha.as_str(), "PIRAHA");
        assert_eq!(LinguisticLense::Sanskrit.as_str(), "SANSKRIT");
    }

    #[test]
    fn test_router_empty_input() {
        let router = LinguisticRouter::new();
        assert_eq!(router.route("", LinguisticLense::Wenyan), "");
        assert_eq!(router.route("", LinguisticLense::Piraha), "");
        assert_eq!(router.route("", LinguisticLense::Sanskrit), "");
    }
}
