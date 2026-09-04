import { useState } from "react";
import {
  UserPlus,
  Pencil,
  Trash2,
  ShieldOff,
  ShieldCheck,
  Loader2,
  X,
  RefreshCw,
  Users,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAuthStore } from "@/stores/auth-store";
import {
  useUsers,
  useCreateUser,
  useUpdateUser,
  useDeleteUser,
  useBlockUser,
} from "@/hooks/use-database";
import type { User } from "@/db/types";

function generatePassword(): string {
  const chars = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%";
  let pass = "";
  const arr = new Uint8Array(12);
  crypto.getRandomValues(arr);
  for (let i = 0; i < 12; i++) pass += chars[arr[i] % chars.length];
  return pass;
}

function formatEnDate(iso: string): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    return `${d.getFullYear()}/${String(d.getMonth() + 1).padStart(2, "0")}/${String(d.getDate()).padStart(2, "0")}`;
  } catch {
    return iso.slice(0, 10);
  }
}

interface DialogProps {
  onClose: () => void;
}

export function AdminManagementDialog({ onClose }: DialogProps) {
  const currentUser = useAuthStore((s) => s.user);
  const { data: users = [], isLoading } = useUsers();
  const createMutation = useCreateUser();
  const updateMutation = useUpdateUser();
  const deleteMutation = useDeleteUser();
  const blockMutation = useBlockUser();

  const [showAdd, setShowAdd] = useState(false);
  const [editItem, setEditItem] = useState<User | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);

  const [addName, setAddName] = useState("");
  const [addEmail, setAddEmail] = useState("");
  const [addPassword, setAddPassword] = useState("");
  const [addError, setAddError] = useState<string | null>(null);

  const [editName, setEditName] = useState("");
  const [editEmail, setEditEmail] = useState("");
  const [editError, setEditError] = useState<string | null>(null);

  const inputClass =
    "w-full px-3 py-2 text-sm border border-gray-300 rounded-lg bg-white text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-800 focus:border-transparent transition";

  const openAdd = () => {
    setAddName("");
    setAddEmail("");
    setAddPassword(generatePassword());
    setAddError(null);
    setShowAdd(true);
  };

  const handleAddSubmit = () => {
    if (!addName.trim()) { setAddError("الاسم مطلوب"); return; }
    if (!addEmail.trim()) { setAddError("البريد الإلكتروني مطلوب"); return; }
    if (!addPassword.trim()) { setAddError("كلمة المرور مطلوبة"); return; }
    createMutation.mutate(
      { full_name: addName.trim(), email: addEmail.trim(), password: addPassword, role: "admin" },
      { onSuccess: () => { setShowAdd(false); }, onError: (e: any) => { setAddError(typeof e === "string" ? e : "حدث خطأ"); } },
    );
  };

  const openEdit = (u: User) => {
    setEditItem(u);
    setEditName(u.full_name);
    setEditEmail(u.email);
    setEditError(null);
  };

  const handleEditSubmit = () => {
    if (!editItem) return;
    if (!editName.trim()) { setEditError("الاسم مطلوب"); return; }
    if (!editEmail.trim()) { setEditError("البريد الإلكتروني مطلوب"); return; }
    updateMutation.mutate(
      { id: editItem.id, data: { full_name: editName.trim(), email: editEmail.trim() } },
      { onSuccess: () => { setEditItem(null); }, onError: (e: any) => { setEditError(typeof e === "string" ? e : "حدث خطأ"); } },
    );
  };

  const handleDelete = () => {
    if (deleteId) {
      deleteMutation.mutate(deleteId, { onSuccess: () => setDeleteId(null) });
    }
  };

  const handleBlock = (u: User) => {
    blockMutation.mutate({ id: u.id, blocked: !u.blocked });
  };

  const thClass = "px-4 py-3 text-right text-xs font-semibold text-slate-600 whitespace-nowrap";
  const tdClass = "px-4 py-3 text-sm text-slate-700 whitespace-nowrap";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-xl shadow-xl w-[780px] max-h-[85vh] flex flex-col animate-scale-in">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-gray-100 flex items-center justify-center">
              <Users className="w-4 h-4 text-slate-500" />
            </div>
            <h2 className="text-lg font-bold text-slate-900">المسؤولون</h2>
          </div>
          <div className="flex items-center gap-2">
            <button onClick={openAdd}
              className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-white bg-slate-800 hover:bg-slate-900 rounded-lg transition">
              <UserPlus className="w-3.5 h-3.5" />
              إضافة مسؤول
            </button>
            <button onClick={onClose} className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition">
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        <div className="flex-1 overflow-auto p-6">

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-6 h-6 text-slate-400 animate-spin" />
        </div>
      ) : (
        <div className="bg-white border border-gray-200 rounded-xl overflow-hidden">
          <table className="w-full" dir="rtl">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className={thClass}>الاسم</th>
                <th className={thClass}>البريد الإلكتروني</th>
                <th className={thClass}>الدور</th>
                <th className={thClass}>الحالة</th>
                <th className={thClass}>تاريخ الإنشاء</th>
                <th className={cn(thClass, "text-center")}>الإجراءات</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {users.map((u) => {
                const isMe = u.id === currentUser?.id;
                return (
                  <tr key={u.id} className="hover:bg-gray-50/80 transition-colors">
                    <td className={tdClass}>{u.full_name}</td>
                    <td className={tdClass}>{u.email}</td>
                    <td className={tdClass}>
                      <span className={cn(
                        "px-2 py-0.5 text-xs font-medium rounded-full",
                        u.role === "admin" ? "bg-violet-100 text-violet-700" : "bg-gray-100 text-gray-600",
                      )}>
                        {u.role === "admin" ? "مسؤول" : "مستخدم"}
                      </span>
                    </td>
                    <td className={tdClass}>
                      <span className={cn(
                        "px-2 py-0.5 text-xs font-medium rounded-full",
                        u.blocked ? "bg-red-100 text-red-700" : "bg-emerald-100 text-emerald-700",
                      )}>
                        {u.blocked ? "محظور" : "نشط"}
                      </span>
                    </td>
                    <td className={tdClass}>{formatEnDate(u.created_at)}</td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <div className="flex items-center gap-1 justify-center">
                        {!isMe && (
                          <>
                            <button onClick={() => openEdit(u)}
                              className="p-1.5 text-slate-400 hover:text-amber-600 hover:bg-amber-50 rounded-md transition-colors"
                              title="تعديل">
                              <Pencil className="w-4 h-4" />
                            </button>
                            <button onClick={() => handleBlock(u)}
                              className={cn(
                                "p-1.5 rounded-md transition-colors",
                                u.blocked
                                  ? "text-slate-400 hover:text-emerald-600 hover:bg-emerald-50"
                                  : "text-slate-400 hover:text-red-600 hover:bg-red-50",
                              )}
                              title={u.blocked ? "إلغاء الحظر" : "حظر"}>
                              {u.blocked ? <ShieldCheck className="w-4 h-4" /> : <ShieldOff className="w-4 h-4" />}
                            </button>
                            <button onClick={() => setDeleteId(u.id)}
                              className="p-1.5 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-md transition-colors"
                              title="حذف">
                              <Trash2 className="w-4 h-4" />
                            </button>
                          </>
                        )}
                      </div>
                    </td>
                  </tr>
                );
              })}
              {users.length === 0 && (
                <tr>
                  <td colSpan={6} className="text-center py-8 text-slate-400 text-sm">لا يوجد مسؤولون</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* Add Dialog */}
      {showAdd && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-xl shadow-xl w-[440px] animate-scale-in">
            <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-bold text-slate-900">إضافة مسؤول جديد</h3>
              <button onClick={() => setShowAdd(false)} className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition">
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">الاسم الكامل</label>
                <input type="text" value={addName} onChange={(e) => setAddName(e.target.value)} className={inputClass} placeholder="الاسم" />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">البريد الإلكتروني</label>
                <input type="email" value={addEmail} onChange={(e) => setAddEmail(e.target.value)} className={inputClass} placeholder="admin@example.com" />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">كلمة المرور</label>
                <div className="flex gap-2">
                  <input type="text" value={addPassword} onChange={(e) => setAddPassword(e.target.value)} className={cn(inputClass, "flex-1")} />
                  <button onClick={() => setAddPassword(generatePassword())}
                    className="p-2 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition shrink-0"
                    title="توليد كلمة مرور">
                    <RefreshCw className="w-4 h-4" />
                  </button>
                </div>
              </div>
              {addError && <p className="text-sm text-red-600">{addError}</p>}
            </div>
            <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
              <button onClick={() => setShowAdd(false)}
                className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition">إلغاء</button>
              <button onClick={handleAddSubmit} disabled={createMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-slate-800 rounded-lg hover:bg-slate-900 disabled:opacity-50 transition">
                {createMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin inline" /> : "إضافة"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit Dialog */}
      {editItem && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-xl shadow-xl w-[440px] animate-scale-in">
            <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-bold text-slate-900">تعديل المسؤول</h3>
              <button onClick={() => setEditItem(null)} className="p-1.5 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition">
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">الاسم الكامل</label>
                <input type="text" value={editName} onChange={(e) => setEditName(e.target.value)} className={inputClass} />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">البريد الإلكتروني</label>
                <input type="email" value={editEmail} onChange={(e) => setEditEmail(e.target.value)} className={inputClass} />
              </div>
              {editError && <p className="text-sm text-red-600">{editError}</p>}
            </div>
            <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200">
              <button onClick={() => setEditItem(null)}
                className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition">إلغاء</button>
              <button onClick={handleEditSubmit} disabled={updateMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-slate-800 rounded-lg hover:bg-slate-900 disabled:opacity-50 transition">
                {updateMutation.isPending ? <Loader2 className="w-4 h-4 animate-spin inline" /> : "حفظ"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation */}
      {deleteId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
          <div className="bg-white rounded-xl shadow-xl p-6 w-[360px] animate-scale-in">
            <h3 className="text-lg font-bold text-slate-900 mb-2">تأكيد الحذف</h3>
            <p className="text-sm text-slate-600 mb-6">هل أنت متأكد من حذف هذا المسؤول؟ لا يمكن التراجع عن هذا الإجراء.</p>
            <div className="flex items-center gap-3 justify-end">
              <button onClick={() => setDeleteId(null)}
                className="px-4 py-2 text-sm font-medium text-slate-600 bg-gray-100 rounded-lg hover:bg-gray-200 transition">إلغاء</button>
              <button onClick={handleDelete} disabled={deleteMutation.isPending}
                className="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 disabled:opacity-50 transition">
                {deleteMutation.isPending ? "جاري الحذف..." : "حذف"}
              </button>
            </div>
          </div>
        </div>
      )}
        </div>
      </div>
    </div>
  );
}
