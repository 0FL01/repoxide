import util from 'node:util';
import { workerData } from 'node:worker_threads';
import pc from 'picocolors';

export const repoxideLogLevels = {
  SILENT: -1, // No output
  ERROR: 0, // error
  WARN: 1, // warn
  INFO: 2, // success, info, log, note
  DEBUG: 3, // debug, trace
} as const;

export type RepoxideLogLevel = (typeof repoxideLogLevels)[keyof typeof repoxideLogLevels];

class RepoxideLogger {
  private level: RepoxideLogLevel = repoxideLogLevels.INFO;

  constructor() {
    this.init();
  }

  init() {
    this.setLogLevel(repoxideLogLevels.INFO);
  }

  setLogLevel(level: RepoxideLogLevel) {
    this.level = level;
  }

  getLogLevel(): RepoxideLogLevel {
    return this.level;
  }

  error(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.ERROR) {
      console.error(pc.red(this.formatArgs(args)));
    }
  }

  warn(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.WARN) {
      console.log(pc.yellow(this.formatArgs(args)));
    }
  }

  success(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.INFO) {
      console.log(pc.green(this.formatArgs(args)));
    }
  }

  info(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.INFO) {
      console.log(pc.cyan(this.formatArgs(args)));
    }
  }

  log(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.INFO) {
      console.log(this.formatArgs(args));
    }
  }

  note(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.INFO) {
      console.log(pc.dim(this.formatArgs(args)));
    }
  }

  debug(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.DEBUG) {
      console.log(pc.blue(this.formatArgs(args)));
    }
  }

  trace(...args: unknown[]) {
    if (this.level >= repoxideLogLevels.DEBUG) {
      console.log(pc.gray(this.formatArgs(args)));
    }
  }

  private formatArgs(args: unknown[]): string {
    return args
      .map((arg) => (typeof arg === 'object' ? util.inspect(arg, { depth: null, colors: true }) : arg))
      .join(' ');
  }
}

export const logger = new RepoxideLogger();

export const setLogLevel = (level: RepoxideLogLevel) => {
  logger.setLogLevel(level);
};

/**
 * Set logger log level from workerData if valid.
 * This is used in worker threads where configuration is passed via workerData.
 */
const isValidLogLevel = (level: number): level is RepoxideLogLevel => {
  return (
    level === repoxideLogLevels.SILENT ||
    level === repoxideLogLevels.ERROR ||
    level === repoxideLogLevels.WARN ||
    level === repoxideLogLevels.INFO ||
    level === repoxideLogLevels.DEBUG
  );
};

export const setLogLevelByWorkerData = () => {
  // Try to get log level from environment variable first (for child_process workers)
  const envLogLevel = process.env.REPOXIDE_LOG_LEVEL;
  if (envLogLevel !== undefined) {
    const logLevel = Number(envLogLevel);
    if (!Number.isNaN(logLevel) && isValidLogLevel(logLevel)) {
      setLogLevel(logLevel);
      return;
    }
  }

  // Fallback to workerData for worker_threads
  if (Array.isArray(workerData) && workerData.length > 1 && workerData[1]?.logLevel !== undefined) {
    const logLevel = workerData[1].logLevel;
    if (isValidLogLevel(logLevel)) {
      setLogLevel(logLevel);
    }
  }
};
