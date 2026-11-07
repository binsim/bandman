export const ADMIN_FILE = "playwright/.auth/admin.json";
export const PARTICIPANT_FILE = "playwright/.auth/participant.json";
export const MEMBER_FILE = "playwright/.auth/member.json";
export const PARTICIPANT_USERNAME = "TestParticipant";
export const MEMBER_USERNAME = "TestMember";

export function isArraySorted(arr: string[]): boolean {
  for (let i = 0; i < arr.length - 1; i++) {
    if (arr[i].localeCompare(arr[i + 1]) > 0) {
      return false; // Found an element that is greater than the next one
    }
  }
  return true; // All elements are in order
}
