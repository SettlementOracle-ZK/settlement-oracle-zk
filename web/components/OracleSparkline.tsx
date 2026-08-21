export type SparkSample = { at: number; value: number };

export function OracleSparkline({
  samples,
  threshold,
}: {
  samples: SparkSample[];
  threshold: number;
}) {
  const width = 720;
  const height = 176;
  const padL = 44;
  const padR = 18;
  const padT = 14;
  const padB = 28;
  const innerW = width - padL - padR;
  const innerH = height - padT - padB;

  const values = samples.map((sample) => sample.value);
  const lo = Math.min(threshold, ...values, 0);
  const hi = Math.max(threshold, ...values, 1);
  const pad = (hi - lo) * 0.14 || 8;
  const yMin = Math.max(0, lo - pad);
  const yMax = hi + pad;
  const ySpan = yMax - yMin || 1;

  const xAt = (index: number) => {
    if (samples.length <= 1) return padL + innerW * 0.08;
    return padL + (index / (samples.length - 1)) * innerW;
  };
  const yAt = (value: number) => padT + innerH - ((value - yMin) / ySpan) * innerH;

  const line = samples
    .map((sample, index) => `${index === 0 ? 'M' : 'L'}${xAt(index).toFixed(1)} ${yAt(sample.value).toFixed(1)}`)
    .join(' ');
  const lastX = xAt(Math.max(samples.length - 1, 0));
  const firstX = xAt(0);
  const baseY = padT + innerH;
  const area = samples.length
    ? `${line} L${lastX.toFixed(1)} ${baseY} L${firstX.toFixed(1)} ${baseY} Z`
    : '';
  const threshY = yAt(threshold);

  return (
    <figure className="sparkline">
      <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Delay index versus trigger while this page is open">
        <title>Session delay index vs trigger threshold</title>
        <line className="sparkline-grid" x1={padL} y1={padT} x2={padL} y2={padT + innerH} />
        <line className="sparkline-grid" x1={padL} y1={padT + innerH} x2={padL + innerW} y2={padT + innerH} />
        <line
          className="sparkline-threshold"
          x1={padL}
          y1={threshY}
          x2={padL + innerW}
          y2={threshY}
        />
        <text className="sparkline-label" x={4} y={threshY + 4}>
          {threshold.toFixed(0)}
        </text>
        {area ? <path className="sparkline-fill" d={area} /> : null}
        {line ? <path className="sparkline-line" d={line} /> : null}
        {samples.map((sample, index) => (
          <circle
            key={`${sample.at}-${index}`}
            className="sparkline-dot"
            cx={xAt(index)}
            cy={yAt(sample.value)}
            r={samples.length - 1 === index ? 3.4 : 2.1}
          />
        ))}
        <text className="sparkline-axis" x={padL} y={height - 8}>
          older
        </text>
        <text className="sparkline-axis" x={padL + innerW} y={height - 8} textAnchor="end">
          now
        </text>
      </svg>
      <figcaption className="sparkline-caption">
        Delay index (min) vs trigger {threshold.toFixed(0)} min · last {samples.length} ticks while
        this page is open · not persisted
      </figcaption>
    </figure>
  );
}
