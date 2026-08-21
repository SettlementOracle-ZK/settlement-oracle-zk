import type { OracleFeed } from '@/lib/types';
import { strikePercents } from '@/lib/trigger';

export function StrikeBoard({
  feed,
  threshold,
  hot,
  hotLabel,
  legendLeft,
  legendRight,
}: {
  feed: OracleFeed;
  threshold: number;
  hot: boolean;
  hotLabel: string;
  legendLeft: string;
  legendRight: string;
}) {
  const { pricePct, thresholdPct } = strikePercents(feed.price, threshold);

  return (
    <div className="strike-board">
      <div className="strike-meta">
        <div>
          <p className="source-note">{feed.symbol}</p>
          <p className={`metric ${feed.stale ? '' : 'metric-live'}`}>{feed.price.toFixed(0)} min</p>
          <p className="lede">
            Confidence ±{feed.conf.toFixed(2)} · age {feed.age_seconds}s
          </p>
        </div>
        <span className="fire-chip" data-hot={hot}>
          {hotLabel}
        </span>
      </div>
      <div className="strike-track" aria-hidden="true">
        <div className="strike-fill" style={{ width: `${Math.max(pricePct, 3)}%` }} />
        <div className="strike-mark" style={{ left: `${thresholdPct}%` }} />
        <div className="strike-spot" style={{ left: `${pricePct}%` }} />
      </div>
      <div className="strike-legend">
        <span>{legendLeft}</span>
        <span>{legendRight}</span>
      </div>
    </div>
  );
}
