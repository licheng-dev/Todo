export interface Todo {
  id: number;
  text: string;
  done: boolean;
  created_at: number;
  completed_at: number | null;
}