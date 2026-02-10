import { describe, it, expect } from 'vitest';
import { parseTimeToSeconds, formatSecondsToTime, formatSecondsToHHMMSS } from './time-parser';

describe('parseTimeToSeconds', () => {
  describe('hours format', () => {
    it('parses hours only', () => {
      expect(parseTimeToSeconds('2h')).toBe(7200);
      expect(parseTimeToSeconds('1h')).toBe(3600);
      expect(parseTimeToSeconds('0h')).toBe(0);
    });

    it('parses decimal hours', () => {
      expect(parseTimeToSeconds('1.5h')).toBe(5400); // 1.5 * 3600
      expect(parseTimeToSeconds('0.5h')).toBe(1800); // 30 minutes
      expect(parseTimeToSeconds('2.25h')).toBe(8100); // 2h 15m
    });

    it('handles whitespace around hours', () => {
      expect(parseTimeToSeconds('2 h')).toBe(7200);
      expect(parseTimeToSeconds('2  h')).toBe(7200);
      expect(parseTimeToSeconds(' 2h ')).toBe(7200);
    });
  });

  describe('minutes format', () => {
    it('parses minutes only', () => {
      expect(parseTimeToSeconds('30m')).toBe(1800);
      expect(parseTimeToSeconds('1m')).toBe(60);
      expect(parseTimeToSeconds('0m')).toBe(0);
    });

    it('parses decimal minutes', () => {
      expect(parseTimeToSeconds('1.5m')).toBe(90); // 1.5 * 60
      expect(parseTimeToSeconds('0.5m')).toBe(30); // 30 seconds
    });

    it('handles whitespace around minutes', () => {
      expect(parseTimeToSeconds('30 m')).toBe(1800);
      expect(parseTimeToSeconds('30  m')).toBe(1800);
      expect(parseTimeToSeconds(' 30m ')).toBe(1800);
    });

    it('does not confuse "m" with "ms"', () => {
      // "m" should match minutes, not milliseconds
      expect(parseTimeToSeconds('30m')).toBe(1800);
      // The regex should not match "ms" as minutes
      expect(parseTimeToSeconds('30ms')).toBe(0); // No match, falls back to 0
    });
  });

  describe('seconds format', () => {
    it('parses seconds only', () => {
      expect(parseTimeToSeconds('45s')).toBe(45);
      expect(parseTimeToSeconds('1s')).toBe(1);
      expect(parseTimeToSeconds('0s')).toBe(0);
    });

    it('parses decimal seconds', () => {
      expect(parseTimeToSeconds('1.5s')).toBe(2); // Rounded
      expect(parseTimeToSeconds('0.5s')).toBe(1); // Rounded
    });

    it('handles whitespace around seconds', () => {
      expect(parseTimeToSeconds('45 s')).toBe(45);
      expect(parseTimeToSeconds('45  s')).toBe(45);
      expect(parseTimeToSeconds(' 45s ')).toBe(45);
    });
  });

  describe('combined formats', () => {
    it('parses hours and minutes', () => {
      expect(parseTimeToSeconds('1h 30m')).toBe(5400); // 3600 + 1800
      expect(parseTimeToSeconds('2h 15m')).toBe(8100); // 7200 + 900
    });

    it('parses hours, minutes, and seconds', () => {
      expect(parseTimeToSeconds('1h 30m 45s')).toBe(5445); // 3600 + 1800 + 45
      expect(parseTimeToSeconds('2h 0m 30s')).toBe(7230); // 7200 + 0 + 30
    });

    it('parses minutes and seconds', () => {
      expect(parseTimeToSeconds('30m 45s')).toBe(1845); // 1800 + 45
      expect(parseTimeToSeconds('1m 30s')).toBe(90); // 60 + 30
    });

    it('handles mixed whitespace in combined formats', () => {
      expect(parseTimeToSeconds('1h30m')).toBe(5400);
      expect(parseTimeToSeconds('1h  30m  45s')).toBe(5445);
      expect(parseTimeToSeconds(' 2h 15m ')).toBe(8100);
    });

    it('handles decimal combined formats', () => {
      expect(parseTimeToSeconds('1.5h 30m')).toBe(7200); // 5400 + 1800
      expect(parseTimeToSeconds('1h 30.5m')).toBe(5430); // 3600 + 1830
    });
  });

  describe('plain number format', () => {
    it('parses plain integer as minutes', () => {
      expect(parseTimeToSeconds('30')).toBe(1800); // 30 * 60
      expect(parseTimeToSeconds('1')).toBe(60);
      expect(parseTimeToSeconds('0')).toBe(0);
    });

    it('parses plain decimal as minutes', () => {
      expect(parseTimeToSeconds('1.5')).toBe(90); // 1.5 * 60
      expect(parseTimeToSeconds('0.5')).toBe(30); // 0.5 * 60
    });

    it('ignores plain number if other units are present', () => {
      // When other units are present, plain numbers are ignored
      expect(parseTimeToSeconds('30m')).toBe(1800);
      // This is "30" as plain number (30 minutes) + "1h" = should be 1h only
      // Actually, the regex will match "1h" first, so plain number match won't apply
      expect(parseTimeToSeconds('1h')).toBe(3600);
    });

    it('handles whitespace around plain numbers', () => {
      expect(parseTimeToSeconds(' 30 ')).toBe(1800);
      expect(parseTimeToSeconds('  1.5  ')).toBe(90);
    });
  });

  describe('case insensitivity', () => {
    it('handles uppercase units', () => {
      expect(parseTimeToSeconds('1H')).toBe(3600);
      expect(parseTimeToSeconds('30M')).toBe(1800);
      expect(parseTimeToSeconds('45S')).toBe(45);
    });

    it('handles mixed case units', () => {
      expect(parseTimeToSeconds('1H 30m')).toBe(5400);
      expect(parseTimeToSeconds('2h 15M 30S')).toBe(8130);
    });
  });

  describe('edge cases', () => {
    it('handles empty string', () => {
      expect(parseTimeToSeconds('')).toBe(0);
    });

    it('handles whitespace only', () => {
      expect(parseTimeToSeconds('   ')).toBe(0);
    });

    it('handles zero values', () => {
      expect(parseTimeToSeconds('0h 0m 0s')).toBe(0);
      expect(parseTimeToSeconds('0')).toBe(0);
    });

    it('handles invalid input gracefully', () => {
      expect(parseTimeToSeconds('invalid')).toBe(0);
      expect(parseTimeToSeconds('abc')).toBe(0);
      expect(parseTimeToSeconds('h m s')).toBe(0);
    });

    it('rounds decimal results to nearest second', () => {
      expect(parseTimeToSeconds('0.9s')).toBe(1); // Rounds up
      expect(parseTimeToSeconds('0.4s')).toBe(0); // Rounds down
      expect(parseTimeToSeconds('1.6s')).toBe(2); // Rounds up
    });

    it('handles very large numbers', () => {
      expect(parseTimeToSeconds('100h')).toBe(360000);
      expect(parseTimeToSeconds('1000m')).toBe(60000);
      expect(parseTimeToSeconds('10000s')).toBe(10000);
    });
  });
});

describe('formatSecondsToTime', () => {
  describe('hours and minutes format', () => {
    it('formats hours and minutes when hours > 0', () => {
      expect(formatSecondsToTime(3600)).toBe('1h 0m');
      expect(formatSecondsToTime(7200)).toBe('2h 0m');
      expect(formatSecondsToTime(5400)).toBe('1h 30m');
      expect(formatSecondsToTime(8100)).toBe('2h 15m');
    });

    it('omits seconds when hours are present', () => {
      expect(formatSecondsToTime(3661)).toBe('1h 1m'); // 1h 1m 1s -> 1h 1m
      expect(formatSecondsToTime(7245)).toBe('2h 0m'); // 2h 0m 45s -> 2h 0m
    });

    it('handles zero minutes with hours', () => {
      expect(formatSecondsToTime(3600)).toBe('1h 0m');
      expect(formatSecondsToTime(7200)).toBe('2h 0m');
    });
  });

  describe('minutes and seconds format', () => {
    it('formats minutes and seconds when no hours', () => {
      expect(formatSecondsToTime(60)).toBe('1m 0s');
      expect(formatSecondsToTime(90)).toBe('1m 30s');
      expect(formatSecondsToTime(1845)).toBe('30m 45s');
    });

    it('handles zero seconds with minutes', () => {
      expect(formatSecondsToTime(120)).toBe('2m 0s');
      expect(formatSecondsToTime(1800)).toBe('30m 0s');
    });
  });

  describe('seconds only format', () => {
    it('formats seconds only when less than a minute', () => {
      expect(formatSecondsToTime(0)).toBe('0s');
      expect(formatSecondsToTime(1)).toBe('1s');
      expect(formatSecondsToTime(30)).toBe('30s');
      expect(formatSecondsToTime(59)).toBe('59s');
    });
  });

  describe('edge cases', () => {
    it('handles large durations', () => {
      expect(formatSecondsToTime(360000)).toBe('100h 0m'); // 100 hours
      expect(formatSecondsToTime(86400)).toBe('24h 0m'); // 24 hours
    });

    it('handles exact boundaries', () => {
      expect(formatSecondsToTime(60)).toBe('1m 0s'); // Exactly 1 minute
      expect(formatSecondsToTime(3600)).toBe('1h 0m'); // Exactly 1 hour
    });
  });
});

describe('formatSecondsToHHMMSS', () => {
  describe('standard formatting', () => {
    it('formats with hours, minutes, seconds', () => {
      expect(formatSecondsToHHMMSS(3661)).toBe('01:01:01'); // 1h 1m 1s
      expect(formatSecondsToHHMMSS(7245)).toBe('02:00:45'); // 2h 0m 45s
      expect(formatSecondsToHHMMSS(5430)).toBe('01:30:30'); // 1h 30m 30s
    });

    it('pads single digits with zeros', () => {
      expect(formatSecondsToHHMMSS(1)).toBe('00:00:01');
      expect(formatSecondsToHHMMSS(60)).toBe('00:01:00');
      expect(formatSecondsToHHMMSS(3600)).toBe('01:00:00');
    });

    it('handles zero', () => {
      expect(formatSecondsToHHMMSS(0)).toBe('00:00:00');
    });
  });

  describe('double-digit values', () => {
    it('formats double-digit values without extra padding', () => {
      expect(formatSecondsToHHMMSS(3723)).toBe('01:02:03'); // 1h 2m 3s
      expect(formatSecondsToHHMMSS(36000)).toBe('10:00:00'); // 10 hours
      expect(formatSecondsToHHMMSS(3660)).toBe('01:01:00'); // 1h 1m
      expect(formatSecondsToHHMMSS(123)).toBe('00:02:03'); // 2m 3s
    });
  });

  describe('large durations', () => {
    it('handles hours >= 100 (no padding limit)', () => {
      expect(formatSecondsToHHMMSS(360000)).toBe('100:00:00'); // 100 hours
      expect(formatSecondsToHHMMSS(359999)).toBe('99:59:59'); // 99h 59m 59s
    });

    it('handles 24-hour boundary', () => {
      expect(formatSecondsToHHMMSS(86400)).toBe('24:00:00'); // Exactly 24 hours
      expect(formatSecondsToHHMMSS(86399)).toBe('23:59:59'); // 23h 59m 59s
    });
  });

  describe('edge cases', () => {
    it('handles exact minute boundaries', () => {
      expect(formatSecondsToHHMMSS(60)).toBe('00:01:00');
      expect(formatSecondsToHHMMSS(120)).toBe('00:02:00');
      expect(formatSecondsToHHMMSS(1800)).toBe('00:30:00');
    });

    it('handles exact hour boundaries', () => {
      expect(formatSecondsToHHMMSS(3600)).toBe('01:00:00');
      expect(formatSecondsToHHMMSS(7200)).toBe('02:00:00');
    });

    it('handles seconds just before boundaries', () => {
      expect(formatSecondsToHHMMSS(59)).toBe('00:00:59');
      expect(formatSecondsToHHMMSS(3599)).toBe('00:59:59');
    });
  });
});

describe('round-trip conversions', () => {
  it('parseTimeToSeconds and formatSecondsToTime are compatible', () => {
    const testCases = [
      '1h 30m',
      '2h 15m',
      '30m 45s',
      '45s'
    ];

    testCases.forEach(input => {
      const seconds = parseTimeToSeconds(input);
      const formatted = formatSecondsToTime(seconds);
      const reparsed = parseTimeToSeconds(formatted);
      expect(reparsed).toBe(seconds);
    });
  });

  it('handles loss of precision in hours format', () => {
    // When formatting with hours, seconds are dropped
    const seconds = parseTimeToSeconds('1h 30m 45s'); // 5445 seconds
    const formatted = formatSecondsToTime(seconds); // "1h 30m" (drops 45s)
    expect(formatted).toBe('1h 30m');

    const reparsed = parseTimeToSeconds(formatted); // 5400 seconds
    expect(reparsed).toBe(5400); // Lost 45 seconds due to formatting
  });
});
