import { PriceServiceConnection } from '@pythnetwork/price-service-client';

import { DEFAULT_HERMES_URL, DEFAULT_PYTH_FEED_ID } from '../constants.js';
import type { PythFeedSnapshot, PriceFeed } from '../types.js';
import { normalizePythPrice } from '../validation.js';

export function mapHermesFeedToPriceFeed(snapshot: PythFeedSnapshot): PriceFeed {
  const price = normalizePythPrice(snapshot.price.price, snapshot.price.expo);
  const conf = normalizePythPrice(snapshot.price.conf, snapshot.price.expo);

  return {
    feedId: snapshot.id,
    price,
    conf,
    expo: snapshot.price.expo,
    publishTime: snapshot.price.publish_time,
  };
}

export class PythHermesClient {
  private readonly connection: PriceServiceConnection;

  constructor(hermesUrl = DEFAULT_HERMES_URL) {
    this.connection = new PriceServiceConnection(hermesUrl);
  }

  async getLatestPriceFeed(feedId = DEFAULT_PYTH_FEED_ID): Promise<PriceFeed> {
    const feeds = await this.connection.getLatestPriceFeeds([feedId]);
    if (!feeds || feeds.length === 0) {
      throw new Error(`No price feed returned for ${feedId}`);
    }

    const raw = feeds[0].getPriceUnchecked();
    return mapHermesFeedToPriceFeed({
      id: feedId,
      price: {
        price: raw.price,
        conf: raw.conf,
        expo: raw.expo,
        publish_time: raw.publishTime,
      },
    });
  }
}
