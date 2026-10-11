#[derive(Debug, Clone)]
pub struct Band {
    pub follow: f64,
    pub review: f64,
}

impl Default for Band {
    fn default() -> Self {
        Self {
            follow: 0.85,
            review: 0.60,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Thresholds {
    pub default_band: Band,
    pub overrides: std::collections::BTreeMap<String, Band>,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            default_band: Band::default(),
            overrides: std::collections::BTreeMap::new(),
        }
    }
}

impl Thresholds {
    pub fn band_for(&self, key: &str) -> Band {
        self.overrides.get(key).cloned().unwrap_or_else(|| self.default_band.clone())
    }

    pub fn classify(&self, key: &str, confidence: f64) -> crate::schema::BandResult {
        let b = self.band_for(key);
        if confidence < 0.0 || confidence > 1.0 || confidence.is_nan() {
            return crate::schema::BandResult::Escalate;
        }
        if confidence >= b.follow {
            crate::schema::BandResult::Follow
        } else if confidence >= b.review {
            crate::schema::BandResult::NeedsReview
        } else {
            crate::schema::BandResult::Escalate
        }
    }
}
