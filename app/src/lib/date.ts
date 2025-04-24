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
