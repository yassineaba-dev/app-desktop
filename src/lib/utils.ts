import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Render the sequential number with its optional "مكرر" (Duplicate) label. */
export function formatSequentialNumber(registrationNumber: string, isDuplicate: boolean): string {
  return isDuplicate ? `${registrationNumber} مكرر` : registrationNumber;
}
