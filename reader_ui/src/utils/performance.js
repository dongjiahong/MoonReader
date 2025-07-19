// Performance optimization utilities for the frontend
import { ref, computed } from "vue";

/**
 * Debounce function to limit API calls
 * @param {Function} func - Function to debounce
 * @param {number} delay - Delay in milliseconds
 * @returns {Function} Debounced function
 */
export function debounce(func, delay) {
  let timeoutId;
  return function (...args) {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => func.apply(this, args), delay);
  };
}

/**
 * Throttle function to limit function execution frequency
 * @param {Function} func - Function to throttle
 * @param {number} limit - Time limit in milliseconds
 * @returns {Function} Throttled function
 */
export function throttle(func, limit) {
  let inThrottle;
  return function (...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Cache for API responses
 */
class APICache {
  constructor(defaultTTL = 5 * 60 * 1000) {
    // 5 minutes default
    this.cache = new Map();
    this.defaultTTL = defaultTTL;
  }

  set(key, data, ttl = this.defaultTTL) {
    const expiresAt = Date.now() + ttl;
    this.cache.set(key, { data, expiresAt });
  }

  get(key) {
    const entry = this.cache.get(key);
    if (!entry) return null;

    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }

    return entry.data;
  }

  clear() {
    this.cache.clear();
  }

  cleanup() {
    const now = Date.now();
    for (const [key, entry] of this.cache.entries()) {
      if (now > entry.expiresAt) {
        this.cache.delete(key);
      }
    }
  }
}

// Global API cache instance
export const apiCache = new APICache();

// Cleanup expired cache entries every 5 minutes
setInterval(() => {
  apiCache.cleanup();
}, 5 * 60 * 1000);

/**
 * Composable for managing loading states
 * @returns {Object} Loading state management
 */
export function useLoading() {
  const loadingStates = ref(new Map());

  const setLoading = (key, isLoading) => {
    if (isLoading) {
      loadingStates.value.set(key, true);
    } else {
      loadingStates.value.delete(key);
    }
  };

  const isLoading = (key) => {
    return loadingStates.value.has(key);
  };

  const isAnyLoading = computed(() => {
    return loadingStates.value.size > 0;
  });

  return {
    setLoading,
    isLoading,
    isAnyLoading,
  };
}

/**
 * Composable for optimized API calls with caching and loading states
 * @param {Function} apiFunction - The API function to call
 * @param {string} cacheKey - Cache key for the API call
 * @param {Object} options - Options for caching and loading
 * @returns {Object} API call management
 */
export function useOptimizedAPI(apiFunction, cacheKey, options = {}) {
  const {
    ttl = 5 * 60 * 1000, // 5 minutes default
    loadingKey = cacheKey,
    useCache = true,
  } = options;

  const { setLoading, isLoading } = useLoading();
  const data = ref(null);
  const error = ref(null);

  const execute = async (...args) => {
    // Check cache first
    if (useCache) {
      const cachedData = apiCache.get(cacheKey);
      if (cachedData) {
        data.value = cachedData;
        return cachedData;
      }
    }

    setLoading(loadingKey, true);
    error.value = null;

    try {
      const result = await apiFunction(...args);
      data.value = result;

      // Cache the result
      if (useCache) {
        apiCache.set(cacheKey, result, ttl);
      }

      return result;
    } catch (err) {
      error.value = err;
      throw err;
    } finally {
      setLoading(loadingKey, false);
    }
  };

  const refresh = async (...args) => {
    // Clear cache and execute
    if (useCache) {
      apiCache.cache.delete(cacheKey);
    }
    return execute(...args);
  };

  return {
    data,
    error,
    execute,
    refresh,
    isLoading: computed(() => isLoading(loadingKey)),
  };
}

/**
 * Optimize large list rendering with virtual scrolling support
 * @param {Array} items - Array of items to render
 * @param {number} itemHeight - Height of each item in pixels
 * @param {number} containerHeight - Height of the container in pixels
 * @returns {Object} Virtual scrolling data
 */
export function useVirtualScrolling(items, itemHeight, containerHeight) {
  const scrollTop = ref(0);

  const visibleItems = computed(() => {
    const startIndex = Math.floor(scrollTop.value / itemHeight);
    const endIndex = Math.min(
      startIndex + Math.ceil(containerHeight / itemHeight) + 1,
      items.value.length
    );

    return {
      startIndex,
      endIndex,
      items: items.value.slice(startIndex, endIndex),
      offsetY: startIndex * itemHeight,
      totalHeight: items.value.length * itemHeight,
    };
  });

  const handleScroll = (event) => {
    scrollTop.value = event.target.scrollTop;
  };

  return {
    visibleItems,
    handleScroll,
    scrollTop,
  };
}

/**
 * Optimize image loading with lazy loading
 * @param {string} src - Image source URL
 * @param {Object} options - Lazy loading options
 * @returns {Object} Image loading state
 */
export function useLazyImage(src, options = {}) {
  const { placeholder = "", threshold = 0.1, rootMargin = "50px" } = options;

  const imageRef = ref(null);
  const isLoaded = ref(false);
  const isInView = ref(false);
  const currentSrc = ref(placeholder);

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          isInView.value = true;
          loadImage();
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold, rootMargin }
  );

  const loadImage = () => {
    if (isLoaded.value || !src) return;

    const img = new Image();
    img.onload = () => {
      currentSrc.value = src;
      isLoaded.value = true;
    };
    img.onerror = () => {
      console.error("Failed to load image:", src);
    };
    img.src = src;
  };

  const observe = (element) => {
    if (element) {
      imageRef.value = element;
      observer.observe(element);
    }
  };

  return {
    currentSrc,
    isLoaded,
    isInView,
    observe,
  };
}

/**
 * Performance monitoring utilities
 */
export class PerformanceMonitor {
  constructor() {
    this.metrics = new Map();
  }

  startTiming(key) {
    this.metrics.set(key, { startTime: performance.now() });
  }

  endTiming(key) {
    const metric = this.metrics.get(key);
    if (metric) {
      metric.endTime = performance.now();
      metric.duration = metric.endTime - metric.startTime;
      return metric.duration;
    }
    return null;
  }

  getMetric(key) {
    return this.metrics.get(key);
  }

  getAllMetrics() {
    return Object.fromEntries(this.metrics);
  }

  logMetrics() {
    console.group("Performance Metrics");
    for (const [key, metric] of this.metrics.entries()) {
      if (metric.duration) {
        console.log(`${key}: ${metric.duration.toFixed(2)}ms`);
      }
    }
    console.groupEnd();
  }

  clear() {
    this.metrics.clear();
  }
}

// Global performance monitor instance
export const performanceMonitor = new PerformanceMonitor();

/**
 * Batch API calls to reduce network requests
 */
export class APIBatcher {
  constructor(batchSize = 10, delay = 100) {
    this.batchSize = batchSize;
    this.delay = delay;
    this.queue = [];
    this.timeoutId = null;
  }

  add(request) {
    return new Promise((resolve, reject) => {
      this.queue.push({ request, resolve, reject });

      if (this.queue.length >= this.batchSize) {
        this.flush();
      } else if (!this.timeoutId) {
        this.timeoutId = setTimeout(() => this.flush(), this.delay);
      }
    });
  }

  async flush() {
    if (this.queue.length === 0) return;

    const batch = this.queue.splice(0, this.batchSize);

    if (this.timeoutId) {
      clearTimeout(this.timeoutId);
      this.timeoutId = null;
    }

    try {
      // Process batch requests
      const results = await Promise.allSettled(
        batch.map(({ request }) => request())
      );

      // Resolve individual promises
      results.forEach((result, index) => {
        const { resolve, reject } = batch[index];
        if (result.status === "fulfilled") {
          resolve(result.value);
        } else {
          reject(result.reason);
        }
      });
    } catch (error) {
      // Reject all promises in case of batch failure
      batch.forEach(({ reject }) => reject(error));
    }

    // Process remaining items if any
    if (this.queue.length > 0) {
      this.timeoutId = setTimeout(() => this.flush(), this.delay);
    }
  }
}

// Global API batcher instance
export const apiBatcher = new APIBatcher();
