import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  incomingCommands,
  outgoingCommands,
  userCommands,
} from "../db/commands";
import type {
  CreateIncomingData,
  UpdateIncomingData,
  CreateOutgoingData,
  UpdateOutgoingData,
  CreateUserData,
  UpdateUserData,
} from "../db/types";

export function useIncoming(page: number, search?: string, exactDate?: string) {
  return useQuery({
    queryKey: ["incoming", page, search, exactDate],
    queryFn: () => incomingCommands.getAll(page, 20, search, exactDate),
    refetchInterval: 15000,
  });
}

export function useIncomingById(id: string) {
  return useQuery({
    queryKey: ["incoming", id],
    queryFn: () => incomingCommands.getById(id),
    enabled: !!id,
  });
}

export function useCreateIncoming() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateIncomingData) => incomingCommands.create(data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["incoming"] }),
  });
}

export function useUpdateIncoming() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateIncomingData }) =>
      incomingCommands.update(id, data),
    onSuccess: (_result, { id }) => {
      qc.invalidateQueries({ queryKey: ["incoming"] });
      qc.invalidateQueries({ queryKey: ["incoming", id] });
    },
  });
}

export function useDeleteIncoming() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => incomingCommands.delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["incoming"] }),
  });
}

export function useOutgoing(page: number, search?: string, exactDate?: string) {
  return useQuery({
    queryKey: ["outgoing", page, search, exactDate],
    queryFn: () => outgoingCommands.getAll(page, 20, search, exactDate),
    refetchInterval: 15000,
  });
}

export function useOutgoingById(id: string) {
  return useQuery({
    queryKey: ["outgoing", id],
    queryFn: () => outgoingCommands.getById(id),
    enabled: !!id,
  });
}

export function useCreateOutgoing() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateOutgoingData) => outgoingCommands.create(data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["outgoing"] }),
  });
}

export function useUpdateOutgoing() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateOutgoingData }) =>
      outgoingCommands.update(id, data),
    onSuccess: (_result, { id }) => {
      qc.invalidateQueries({ queryKey: ["outgoing"] });
      qc.invalidateQueries({ queryKey: ["outgoing", id] });
    },
  });
}

export function useDeleteOutgoing() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => outgoingCommands.delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["outgoing"] }),
  });
}

export function useUsers() {
  return useQuery({
    queryKey: ["users"],
    queryFn: () => userCommands.getAll(),
  });
}

export function useUserById(id: string) {
  return useQuery({
    queryKey: ["users", id],
    queryFn: () => userCommands.getById(id),
    enabled: !!id,
  });
}

export function useCreateUser() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: CreateUserData) => userCommands.create(data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["users"] }),
  });
}

export function useUpdateUser() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateUserData }) =>
      userCommands.update(id, data),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["users"] }),
  });
}

export function useDeleteUser() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => userCommands.delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["users"] }),
  });
}

export function useBlockUser() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, blocked }: { id: string; blocked: boolean }) =>
      userCommands.block(id, blocked),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["users"] }),
  });
}

