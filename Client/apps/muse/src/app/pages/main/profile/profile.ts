import {Component, EventEmitter, Output, OnInit} from '@angular/core';
import { MatFormField, MatInput, MatLabel} from '@angular/material/input';
import {FormControl, FormGroup, FormsModule, ReactiveFormsModule, Validators} from '@angular/forms';
import {MatButton} from '@angular/material/button';
import { faUserCircle, faSave, faExclamationTriangle, faTrash } from '@fortawesome/free-solid-svg-icons';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import { fetchWithAuth } from '../../../app';
import { environment } from '../../../../environments/environment';
import { NgClass } from '@angular/common';

@Component({
  selector: 'app-profile',
  imports: [
    MatFormField,
    MatLabel,
    MatInput,
    FormsModule,
    ReactiveFormsModule,
    MatButton,
    FaIconComponent,
    NgClass,
  ],
  templateUrl: './profile.html',
  styleUrl: './profile.scss'
})
export class Profile implements OnInit {
  protected readonly userinfo = new FormGroup({
    username: new FormControl('', [Validators.required]),
    email: new FormControl('', [Validators.required, Validators.email])
  });
  protected readonly faUserCircle = faUserCircle;
  protected readonly faSave = faSave;
  protected readonly faExclamationTriangle = faExclamationTriangle;
  protected readonly faTrash = faTrash;

  public message: string | null = null;
  public messageType: 'success' | 'error' | null = null;
  public showDeleteDialog = false;
  public deletePassword = '';
  public deleteMessage: string | null = null;
  public deleteMessageType: 'success' | 'error' | null = null;

  get isDirty(): boolean {
    return this.userinfo.dirty;
  }

  async ngOnInit() {
    const token = localStorage.getItem('authToken');
    if (!token) return;
    const response = await fetchWithAuth(`${environment.apiUrl}/api/user`, {
      headers: { Authorization: `Bearer ${token}` }
    });
    if (response.ok) {
      const data = await response.json();
      if (data && data.data) {
        this.userinfo.patchValue({
          username: data.data.username,
          email: data.data.email
        });
      }
    }
  }

  async onSubmit() {
    if (this.userinfo.invalid) return;
    const token = localStorage.getItem('authToken');
    if (!token) return;
    const body = {
      username: this.userinfo.value.username,
      email: this.userinfo.value.email
    };
    const response = await fetchWithAuth(`${environment.apiUrl}/api/user`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`
      },
      body: JSON.stringify(body)
    });
    if (response.ok) {
      this.message = 'Profile updated successfully!';
      this.messageType = 'success';
    } else {
      let errorMsg = 'Failed to update profile.';
      try {
        const errorData = await response.json();
        if (errorData && errorData.message) {
          errorMsg = errorData.message;
        }
      } catch {}
      this.message = errorMsg;
      this.messageType = 'error';
    }
  }

  openDeleteDialog() {
    this.showDeleteDialog = true;
    this.deletePassword = '';
    this.deleteMessage = null;
    this.deleteMessageType = null;
  }

  closeDeleteDialog() {
    this.showDeleteDialog = false;
    this.deletePassword = '';
    this.deleteMessage = null;
    this.deleteMessageType = null;
  }

  async confirmDeleteAccount() {
    const token = localStorage.getItem('authToken');
    if (!token || !this.deletePassword) {
      this.deleteMessage = 'Password is required.';
      this.deleteMessageType = 'error';
      return;
    }
    const response = await fetchWithAuth(`${environment.apiUrl}/api/user/delete`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`
      },
      body: JSON.stringify({ password: this.deletePassword })
    });
    if (response.ok) {
      this.deleteMessage = 'Account deleted successfully.';
      this.deleteMessageType = 'success';
      // Optionally, log out or redirect
    } else {
      let errorMsg = 'Failed to delete account.';
      try {
        const errorData = await response.json();
        if (errorData && errorData.message) {
          errorMsg = errorData.message;
        }
      } catch {}
      this.deleteMessage = errorMsg;
      this.deleteMessageType = 'error';
    }
  }
}
