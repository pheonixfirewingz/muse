import {ChangeDetectionStrategy, Component, inject, signal} from '@angular/core';
import {MatCardModule} from '@angular/material/card';
import {MatButtonModule} from '@angular/material/button';
import {MatFormFieldModule} from '@angular/material/form-field';
import {
  FormControl,
  FormGroup,
  ReactiveFormsModule,
  Validators
} from '@angular/forms';
import {MatInput} from '@angular/material/input';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import  {faEye, faEyeSlash} from '@fortawesome/free-solid-svg-icons';
import {Router} from '@angular/router';
import {environment} from '../../../environments/environment';
import { fetchWithAuth } from '../../app';

@Component({
  selector: 'app-login',
  imports: [MatCardModule, MatButtonModule, MatFormFieldModule, ReactiveFormsModule, MatInput, FaIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './login.html',
  styleUrl: './login.css'
})
export class Login {
  private router = inject(Router);
  public static readonly supportedAPIVersion: number[] = [0,1,0];
  protected isRegisterMode:boolean = false;
  hide = signal(true);

  clickEvent(event: MouseEvent) {
    this.hide.set(!this.hide());
    event.stopPropagation();
  }

  getVisIcon() {
    return this.hide() ? faEyeSlash : faEye;
  }

  protected readonly login = new FormGroup({
    username: new FormControl('local_checks',[Validators.required]),
    password: new FormControl('tuh6y6Q8N5q*tF4^vhx&@fPE8s',[Validators.required, Validators.minLength(12)]),
  });

  protected readonly register = new FormGroup({
    username: new FormControl('', [Validators.required]),
    email: new FormControl('', [Validators.required, Validators.email]),
    password: new FormControl('', [Validators.required, Validators.minLength(12)]),
    confirmPassword: new FormControl('', [Validators.required])
  });

  toggleMode():void {
    this.isRegisterMode = !this.isRegisterMode;
  }

  get username():string {
    return (this.isRegisterMode ? this.register.get('username')?.value : this.login.get('username')?.value) as string;
  }

  get password():string {
    return (this.isRegisterMode ? this.register.get('password')?.value : this.login.get('password')?.value) as string;
  }

  get email():string {
    return this.register.get('email')?.value as string;
  }

  get confirmPassword():string {
    return this.register.get('confirmPassword')?.value as string;
  }

  async sendLoginRequest(): Promise<void> {
    if (!this.login.valid) {
      console.log('Form is invalid, not sending request');
      return;
    }
    try {
      const response = await fetchWithAuth(`${environment.apiUrl}/api/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username: this.username,
          password: this.password,
        }),
      }, this.router);
      const json = await response.json();

      if (json.success) {
        console.info(json.message);
        localStorage.setItem('authToken',json.token)
        await this.router.navigate(['/app']);
      } else {
        console.error('Login failed:', json.message);
      }
    } catch (error) {
      console.error('Login error:', error);
    }
  }

  async signupRequest(): Promise<void> {
    if (!this.register.valid) {
      console.log('Form is invalid, not sending request');
      return;
    }
    try {
      const response = await fetchWithAuth(`${environment.apiUrl}/api/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username: this.username,
          email: this.email,
          password: this.password,
          confirmPassword: this.confirmPassword
        }),
      }, this.router);
      const json = await response.json();

      if (json.success) {
        this.isRegisterMode = false;
      } else {
        console.error('Registration failed:', json.message);
      }
    } catch (error) {
      console.error('Registration error:', error);
    }
  }
}
