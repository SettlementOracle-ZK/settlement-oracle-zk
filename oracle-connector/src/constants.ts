/** MVP feed: SOL/USD on Pyth (devnet + mainnet) */
export const DEFAULT_PYTH_FEED_ID =
  '0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d';

export const DEFAULT_HERMES_URL = 'https://hermes.pyth.network';

/** Reject oracle data older than this (seconds). */
export const MAX_STALENESS_SECONDS = 60;

/** Reject when confidence / |price| exceeds this ratio. */
export const MAX_CONFIDENCE_RATIO = 0.05;
