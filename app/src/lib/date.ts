/**
 * Returns a short, human-readable description of how long ago a given timestamp was.
 * Examples: "Just now", "Today", "Yesterday", "This week", "Last week", "12 days ago"
 *
 * @param timestamp - A date input (timestamp, string, or Date object)
 * @returns A relative time description string
 */
export const formatRelativeDate = (
  timestamp: number | string | Date,
): string => {
  const inputDate = new Date(timestamp);
  const now = new Date();

  const msInMinute = 60 * 1000;
  const msInDay = 24 * 60 * 60 * 1000;

  const timeDiff = now.getTime() - inputDate.getTime();
  const minutesDiff = Math.floor(timeDiff / msInMinute);
  const daysDiff = Math.floor(timeDiff / msInDay);

  const isSameDay = (d1: Date, d2: Date) =>
    d1.toDateString() === d2.toDateString();

  if (minutesDiff < 3) {
    return "Just now";
  } else if (isSameDay(inputDate, now)) {
    return "Today";
  } else if (isSameDay(inputDate, new Date(now.getTime() - msInDay))) {
    return "Yesterday";
  } else if (daysDiff < 7) {
    return "This week";
  } else if (daysDiff < 14) {
    return "Last week";
  } else {
    return `${daysDiff} days ago`;
  }
};

/**
 * Converts a timestamp into a readable formatted string with date and time.
 * Example output: "Apr 24, 2025, 11:58 AM"
 *
 * @param timestamp - A date input (timestamp, string, or Date object)
 * @returns A nicely formatted date string
 */
export const readableDate = (timestamp: number | string | Date) => {
  const date = new Date(timestamp);
  return `${date.toLocaleDateString("en-US", {
    month: "short", // e.g. "Apr"
    day: "numeric", // e.g. "24"
    year: "numeric", // e.g. "2025"
  })}, ${date.toLocaleTimeString("en-US", {
    hour: "numeric", // e.g. "11"
    minute: "2-digit", // e.g. "58"
    hour12: true, // "AM"/"PM"
  })}`;
};

/**
 * Returns an ISO timestamp representing the start date based on a given time range (e.g., "7d" => 7 days ago).
 * This is used to filter records created within the last N days.
 *
 * @param timePeriod - A string like "7d", "30d", or "90d"
 * @returns ISO string timestamp representing the lower bound date
 */
export const getStartDateFromTimePeriod = (timePeriod: string): string => {
  const now = new Date();
  const daysAgo = parseInt(timePeriod.replace("d", ""), 10);
  now.setDate(now.getDate() - daysAgo);
  return now.toISOString();
};
