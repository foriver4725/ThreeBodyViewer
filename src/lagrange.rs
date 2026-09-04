use crate::model::Star;
use nalgebra::Vector3;
pub(crate) struct LagrangePoint {
    pub(crate) name: &'static str,
    pub(crate) position: Vector3<f64>,
}

pub(crate) struct LagrangePointFactory;

impl LagrangePointFactory {
    /// 2つの主星から5つのラグランジュ点を生成する。
    ///
    /// 円制限三体問題（主星同士は円運動し、置く物体の質量は無視できる）を
    /// 仮定している。L1〜L3は数値計算、L4・L5は正三角形から求める。
    pub(crate) fn create(primary: &Star, secondary: &Star) -> [LagrangePoint; 5] {
        let relative = secondary.position - primary.position;
        let distance = relative.norm();
        assert!(distance > f64::EPSILON, "主星同士の位置が重なっています");
        assert!(
            primary.mass > 0.0 && secondary.mass > 0.0,
            "主星の質量は正の値にしてください"
        );

        let x_axis = relative / distance;
        // 現在はXY平面上の運動を扱うため、x軸を90度回した方向をy軸にする。
        let y_axis = Vector3::new(-x_axis.y, x_axis.x, 0.0);
        let total_mass = primary.mass + secondary.mass;
        let mu = secondary.mass / total_mass;
        let center =
            (primary.position * primary.mass + secondary.position * secondary.mass) / total_mass;

        // 重心を原点、主星間距離を1とした回転座標系でL1〜L3を解く。
        let l1_x = Self::solve_collinear(mu, -mu, 1.0 - mu);
        let l2_x = Self::solve_collinear(mu, 1.0 - mu, 2.0);
        let l3_x = Self::solve_collinear(mu, -2.0, -mu);

        let point = |name, x: f64, y: f64| LagrangePoint {
            name,
            position: center + (x_axis * x + y_axis * y) * distance,
        };

        [
            point("L1", l1_x, 0.0),
            point("L2", l2_x, 0.0),
            point("L3", l3_x, 0.0),
            point("L4", 0.5 - mu, 3.0_f64.sqrt() / 2.0),
            point("L5", 0.5 - mu, -3.0_f64.sqrt() / 2.0),
        ]
    }

    // x方向の有効力が0になる場所を二分法で探す。
    fn solve_collinear(mu: f64, left: f64, right: f64) -> f64 {
        // 主星そのもの（区間端）では式が発散するため、わずかに内側から探す。
        let epsilon = 1.0e-9;
        let mut low = left + epsilon;
        let mut high = right - epsilon;

        for _ in 0..100 {
            let middle = (low + high) / 2.0;
            if Self::effective_force(mu, low).signum() == Self::effective_force(mu, middle).signum()
            {
                low = middle;
            } else {
                high = middle;
            }
        }

        (low + high) / 2.0
    }

    // 正規化した回転座標系における、x方向の重力と遠心力の合計。
    fn effective_force(mu: f64, x: f64) -> f64 {
        let primary_offset = x + mu;
        let secondary_offset = x - (1.0 - mu);
        x - (1.0 - mu) * primary_offset / primary_offset.abs().powi(3)
            - mu * secondary_offset / secondary_offset.abs().powi(3)
    }
}
